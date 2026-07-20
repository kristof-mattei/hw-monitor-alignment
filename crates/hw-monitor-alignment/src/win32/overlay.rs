#![expect(
    unused_results,
    unused_must_use,
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "Win32 FFI requires explicit pointer/integer casts throughout"
)]
#![expect(
    clippy::multiple_unsafe_ops_per_block,
    reason = "Win32 interop requires batching several unsafe calls"
)]
#![expect(
    unsafe_op_in_unsafe_fn,
    reason = "wndproc is already declared unsafe, and redundant inner blocks add noise"
)]

mod gdiplus;
mod window;

use std::sync::OnceLock;

use windows::Win32::libloaderapi::GetModuleHandleW;
use windows::Win32::minwindef::{LPARAM, WPARAM};
use windows::Win32::windef::{COLORREF, HDC, HGDIOBJ, HICON, HWND};
use windows::Win32::wingdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush, DeleteDC, DeleteObject,
    SRCCOPY, SelectObject,
};
use windows::Win32::winuser::{
    BeginPaint, DestroyWindow, EndPaint, InvalidateRect, PAINTSTRUCT, PostMessageW,
    RegisterClassExW, SetFocus, WM_CLOSE, WNDCLASSEXW,
};
use windows_core::{HSTRING, PCWSTR};

use self::gdiplus::{GdiplusStartup, GdiplusStartupInput};
use crate::monitor::Monitor;
use crate::state::SharedSession;

const CLASS_NAME: &str = "HwMonitorAlignmentOverlay";

/// `RGB` macro equivalent (windows-sys does not export one).
const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

/// Render into an off-screen bitmap and blit it to the window in one operation, so a
/// repaint never shows a half-drawn (flashing) frame.
fn paint_double_buffered<F: FnOnce(HDC)>(hwnd: HWND, w: i32, h: i32, draw: F) {
    // SAFETY: standard BeginPaint -> off-screen render -> BitBlt -> EndPaint.
    unsafe {
        let mut paint_struct = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &raw mut paint_struct);

        let mem = CreateCompatibleDC(Some(hdc));

        let bmp = CreateCompatibleBitmap(hdc, w, h);

        let old = SelectObject(mem, HGDIOBJ(bmp.0));

        draw(mem);

        BitBlt(hdc, 0, 0, w, h, Some(mem), 0, 0, SRCCOPY);

        SelectObject(mem, old);

        DeleteObject(HGDIOBJ(bmp.0));

        DeleteDC(mem);

        EndPaint(hwnd, &raw const paint_struct);
    }
}

fn invalidate_all(session: &SharedSession) {
    let hwnds: Vec<HWND> = {
        let lock = session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        lock.hwnd_list.clone()
    };

    for val in hwnds {
        // SAFETY: val was stored by create_overlays, window may already be gone.
        // we still pass in `Some()`, as passing in `None` redraws all windows.
        unsafe {
            InvalidateRect(Some(val), None, false);
        }
    }
}

fn stop_all(session: &SharedSession) {
    let hwnds: Vec<HWND> = {
        let mut lock = session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        lock.stop_requested = true;

        lock.version = lock.version.wrapping_add(1);

        lock.hwnd_list.clone()
    };

    for val in hwnds {
        // SAFETY: PostMessageW is safe even if the window was already destroyed.
        unsafe {
            PostMessageW(Some(val), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
}

/// Extract the `(device_name, x, working_y)` for every monitor in the `session`.
#[expect(
    dead_code,
    reason = "used by the Apply flow of the control box, which is temporarily removed"
)]
fn collect_positions(session: &SharedSession) -> Vec<(Box<str>, i32, i32)> {
    let lock = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    lock.monitors
        .iter()
        .zip(lock.working_y.iter())
        .map(|(m, &y)| (m.device_name.clone(), m.x, y))
        .collect()
}

static CLASS_REGISTERED: OnceLock<bool> = OnceLock::new();
static GDIP_TOKEN: OnceLock<usize> = OnceLock::new();

fn gdip_init() {
    GDIP_TOKEN.get_or_init(|| {
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            DebugEventCallback: 0,
            SuppressBackgroundThread: false.into(),
            SuppressExternalCodecs: false.into(),
        };

        let mut token: usize = 0;

        // SAFETY: input is fully initialized
        unsafe {
            GdiplusStartup(&raw mut token, &raw const input, std::ptr::null_mut());
        }

        token
    });
}

fn ensure_class_registered() -> bool {
    *CLASS_REGISTERED.get_or_init(|| {
        let class_wide = HSTRING::from(CLASS_NAME);

        // SAFETY: GetModuleHandleW(null) always succeeds
        let h_module = unsafe { GetModuleHandleW(PCWSTR::null()) };

        // SAFETY: API call
        let bg = unsafe { CreateSolidBrush(rgb(255, 255, 255)) };

        let wc = WNDCLASSEXW {
            cbSize: u32::try_from(size_of::<WNDCLASSEXW>()).unwrap_or(0),
            style: 0,
            lpfnWndProc: Some(window::overlay_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_module,
            hIcon: HICON::default(),
            hCursor: HICON::default(),
            hbrBackground: bg,
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR::from_raw(class_wide.as_ptr()),
            hIconSm: HICON::default(),
        };

        // SAFETY: wc is fully initialised.
        unsafe { RegisterClassExW(&raw const wc) }.0 != 0
    })
}

pub fn create_overlays(session: &SharedSession) {
    gdip_init();

    if !ensure_class_registered() {
        return;
    }

    // capture starting layout
    let monitors_snapshot: std::sync::Arc<[Monitor]> = {
        let lock = session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        std::sync::Arc::clone(&lock.monitors)
    };

    let mut hwnd_list: Vec<HWND> = Vec::new();

    for (index, monitor) in monitors_snapshot.iter().enumerate() {
        let overlay = match window::create_overlay_window(session, index, monitor) {
            Ok(overlay) => overlay,
            Err(_error) => continue,
        };

        hwnd_list.push(overlay);

        // SAFETY: keep keyboard focus on the overlay (the arrow-key nudge target)
        unsafe {
            SetFocus(Some(overlay));
        }
    }

    let mut lock = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    lock.hwnd_list = hwnd_list;
}

pub fn destroy_remaining_overlays(session: &SharedSession) {
    let hwnds: Vec<HWND> = {
        let lock = session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        lock.hwnd_list.clone()
    };

    for val in hwnds {
        // SAFETY: DestroyWindow is safe to call with any handle value.
        unsafe {
            DestroyWindow(val);
        }
    }
}
