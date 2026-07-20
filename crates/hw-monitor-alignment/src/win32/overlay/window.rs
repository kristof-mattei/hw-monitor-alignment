//! The fullscreen, borderless overlay window — one per monitor. It paints the
//! white background, the alignment lines and the cursor readout, and owns the
//! arrow-key nudging.

use std::rc::Rc;

use windows::Win32::libloaderapi::GetModuleHandleW;
use windows::Win32::minwindef::{LPARAM, LRESULT, WPARAM};
use windows::Win32::windef::{HDC, HGDIOBJ, HWND, RECT};
use windows::Win32::wingdi::{
    CreateSolidBrush, DEFAULT_GUI_FONT, DeleteObject, GetStockObject, SelectObject, SetBkMode,
    SetTextColor, TRANSPARENT,
};
use windows::Win32::winuser::{
    BringWindowToTop, CREATESTRUCTW, CW_USEDEFAULT, CreateWindowExW, DT_RIGHT, DT_SINGLELINE,
    DT_TOP, DefWindowProcW, DestroyWindow, DrawTextW, FillRect, GWLP_USERDATA, GetWindowLongPtrW,
    InvalidateRect, SW_SHOW, SWP_NOZORDER, SetFocus, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    VK_DOWN, VK_ESCAPE, VK_NEXT, VK_PRIOR, VK_UP, WM_CLOSE, WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN,
    WM_MOUSEMOVE, WM_NCCREATE, WM_PAINT, WM_SYSKEYDOWN, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};
use windows_core::{HSTRING, PCWSTR, WIN32_ERROR};

use super::gdiplus::{
    GdipCreateFromHDC, GdipCreatePen1, GdipDeleteGraphics, GdipDeletePen, GdipDrawLine,
    GdipSetSmoothingMode, GpGraphics, GpPen, SmoothingModeAntiAlias, SmoothingModeNone, UnitPixel,
};
use super::{CLASS_NAME, invalidate_all, paint_double_buffered, rgb, stop_all};
use crate::monitor::Monitor;
use crate::state::SharedSession;

/// GDI+ ARGB: 0xAARRGGBB (fully opaque when alpha = 0xFF).
const fn argb(r: u8, g: u8, b: u8) -> u32 {
    0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Stored in the fullscreen overlay window's `GWLP_USERDATA`. The overlay only
/// paints the background, alignment lines and cursor readout.
struct OverlayData {
    monitor_index: usize,
    monitor_w: i32,
    monitor_h: i32,
    session: SharedSession,
    cursor_x: i32,
    cursor_y: i32,
}

fn move_monitor(data: &OverlayData, delta: i32) {
    {
        let mut lock = data
            .session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let index = data.monitor_index;
        if !lock.monitors[index].primary {
            lock.working_y[index] = lock.working_y[index].saturating_add(delta);
            lock.version = lock.version.wrapping_add(1);
        }
    }

    invalidate_all(&data.session);
}

fn handle_key(data: &mut OverlayData, hwnd: HWND, vk: u32) {
    if vk == VK_ESCAPE {
        stop_all(&data.session);
    } else if vk == VK_UP {
        move_monitor(data, 1);
    } else if vk == VK_DOWN {
        move_monitor(data, -1);
    } else if vk == VK_PRIOR {
        move_monitor(data, 10);
    } else if vk == VK_NEXT {
        move_monitor(data, -10);
    } else {
        // ...
    }

    // SAFETY: SetFocus is safe on the window receiving the key message.
    unsafe {
        SetFocus(Some(hwnd));
    }
}

struct PaintState {
    primary_h: i32,
    pos_y: i32,
    show_diagonal: bool,
    show_horizontal: bool,
    show_cursor: bool,
    antialiasing: bool,
    line_spacing: i32,
    line_thickness: i32,
}

fn read_paint_state(data: &OverlayData) -> PaintState {
    let lock = data
        .session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let primary_h = lock
        .monitors
        .iter()
        .find(|m| m.primary)
        .map_or(1080, |m| m.height as i32);

    PaintState {
        primary_h,
        pos_y: lock.working_y[data.monitor_index],
        show_diagonal: lock.settings.show_diagonal_lines,
        show_horizontal: lock.settings.show_horizontal_lines,
        show_cursor: lock.settings.show_cursor_position,
        antialiasing: lock.settings.antialiasing,
        line_spacing: lock.settings.line_spacing,
        line_thickness: lock.settings.line_thickness,
    }
}

fn paint(hdc: HDC, data: &OverlayData) {
    let ps = read_paint_state(data);

    let w = data.monitor_w;
    let h = data.monitor_h;

    // SAFETY: hdc valid between BeginPaint/EndPaint and all GDI objects are released before return.
    unsafe {
        // This allocates
        let white = CreateSolidBrush(rgb(255, 255, 255));

        let rc = RECT {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
        };

        FillRect(hdc, &raw const rc, white);

        DeleteObject(HGDIOBJ(white.0));

        let graphics = {
            let mut graphics: *mut GpGraphics = std::ptr::null_mut();

            GdipCreateFromHDC(hdc, &raw mut graphics);

            graphics
        };

        let smoothing = if ps.antialiasing {
            SmoothingModeAntiAlias
        } else {
            SmoothingModeNone
        };

        GdipSetSmoothingMode(graphics, smoothing);

        // Diagonal lines
        if ps.show_diagonal {
            let mut gray_pen: *mut GpPen = std::ptr::null_mut();

            GdipCreatePen1(argb(180, 180, 180), 1.0, UnitPixel, &raw mut gray_pen);

            GdipDrawLine(graphics, gray_pen, 0.0, 0.0, w as f32, h as f32);
            GdipDrawLine(graphics, gray_pen, w as f32, 0.0, 0.0, h as f32);

            GdipDeletePen(gray_pen);
        }

        // Horizontal alignment lines
        if ps.show_horizontal {
            let short = w / 5;

            let mid = ps.primary_h / 2;

            let y_over = mid - ps.line_spacing - ps.pos_y;
            let y_center = mid - ps.pos_y;
            let y_under = mid + ps.line_spacing - ps.pos_y;

            let mut black_pen: *mut GpPen = std::ptr::null_mut();

            GdipCreatePen1(
                argb(0, 0, 0),
                ps.line_thickness as f32,
                UnitPixel,
                &raw mut black_pen,
            );

            if y_center >= 0 && y_center <= h {
                GdipDrawLine(
                    graphics,
                    black_pen,
                    0.0,
                    y_center as f32,
                    w as f32,
                    y_center as f32,
                );
            }

            if y_over >= 0 && y_over <= h {
                GdipDrawLine(
                    graphics,
                    black_pen,
                    0.0,
                    y_over as f32,
                    short as f32,
                    y_over as f32,
                );

                GdipDrawLine(
                    graphics,
                    black_pen,
                    (w - short) as f32,
                    y_over as f32,
                    w as f32,
                    y_over as f32,
                );
            }

            if y_under >= 0 && y_under <= h {
                GdipDrawLine(
                    graphics,
                    black_pen,
                    0.0,
                    y_under as f32,
                    short as f32,
                    y_under as f32,
                );

                GdipDrawLine(
                    graphics,
                    black_pen,
                    (w - short) as f32,
                    y_under as f32,
                    w as f32,
                    y_under as f32,
                );
            }

            GdipDeletePen(black_pen);
        }

        GdipDeleteGraphics(graphics);

        // Cursor position overlay
        if ps.show_cursor {
            let text = format!("{}, {}", data.cursor_x, data.cursor_y);

            let wide: Vec<u16> = text.encode_utf16().collect();

            // stock objects don't need to be deleted
            let hfont = GetStockObject(DEFAULT_GUI_FONT.cast_signed());

            let old_font = SelectObject(hdc, hfont);

            SetBkMode(hdc, TRANSPARENT.cast_signed());
            SetTextColor(hdc, rgb(60, 60, 60));

            let mut rc = RECT {
                left: w - 160,
                top: 8,
                right: w - 8,
                bottom: 28,
            };

            DrawTextW(
                hdc,
                PCWSTR::from_raw(wide.as_ptr()),
                i32::try_from(wide.len()).expect("cursor text length fits i32"),
                &raw mut rc,
                DT_RIGHT | DT_TOP | DT_SINGLELINE,
            );

            SelectObject(hdc, old_font);
        }
    }
}

pub(super) unsafe extern "system" fn overlay_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            let cs = &*(lparam.0 as *const CREATESTRUCTW);

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, cs.lpCreateParams as isize);

            DefWindowProcW(hwnd, msg, wparam, lparam)
        },

        WM_ERASEBKGND => LRESULT(1), // the double-buffered WM_PAINT repaints fully, so skip the erase flash here

        WM_PAINT => {
            let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const OverlayData;

            if pointer.is_null() {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }

            let data = &*pointer;

            paint_double_buffered(hwnd, data.monitor_w, data.monitor_h, |mem| paint(mem, data));

            LRESULT(0)
        },

        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayData;

            if pointer.is_null() {
                return LRESULT(0);
            }

            handle_key(
                &mut *pointer,
                hwnd,
                u32::try_from(wparam.0 & 0xFFFF).expect("masked to 16 bits"),
            );

            LRESULT(0)
        },

        WM_MOUSEMOVE => {
            let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayData;

            if pointer.is_null() {
                return LRESULT(0);
            }

            let data = &mut *pointer;

            data.cursor_x = i32::from((lparam.0 & 0xFFFF) as i16);
            data.cursor_y = i32::from(((lparam.0 >> 16) & 0xFFFF) as i16);

            let show = {
                let lock = data
                    .session
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);

                lock.settings.show_cursor_position
            };

            if show {
                InvalidateRect(Some(hwnd), None, false);
            }

            LRESULT(0)
        },

        WM_CLOSE => {
            DestroyWindow(hwnd);

            LRESULT(0)
        },

        WM_DESTROY => {
            let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayData;

            if !pointer.is_null() {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);

                drop(Box::from_raw(pointer));
            }

            LRESULT(0)
        },

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Create and show the fullscreen overlay for `monitor`, positioned at its
/// virtual-screen coordinates.
pub(super) fn create_overlay_window(
    session: &SharedSession,
    index: usize,
    monitor: &Monitor,
) -> Result<HWND, WIN32_ERROR> {
    let mw = monitor.width as i32;
    let mh = monitor.height as i32;

    let class_wide = HSTRING::from(CLASS_NAME);
    let title_wide = HSTRING::from("HwMonitorAlignment Overlay");

    let data = Box::new(OverlayData {
        monitor_index: index,
        monitor_w: mw,
        monitor_h: mh,
        session: Rc::clone(session),
        cursor_x: 0,
        cursor_y: 0,
    });

    let data_ptr = Box::into_raw(data);

    // SAFETY: api call
    let overlay = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            PCWSTR::from_raw(class_wide.as_ptr()),
            PCWSTR::from_raw(title_wide.as_ptr()),
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            mw,
            mh,
            None,
            None,
            Some(GetModuleHandleW(PCWSTR::null())),
            Some(data_ptr.cast()),
        )
    };

    if overlay.0.is_null() {
        return Err(WIN32_ERROR::from_thread());
    }

    // SAFETY: reposition the overlay to the monitor's virtual-screen coordinates, then show.
    unsafe {
        SetWindowPos(overlay, None, monitor.x, monitor.y, mw, mh, SWP_NOZORDER);
        ShowWindow(overlay, SW_SHOW.cast_signed());
        BringWindowToTop(overlay);
    }

    Ok(overlay)
}
