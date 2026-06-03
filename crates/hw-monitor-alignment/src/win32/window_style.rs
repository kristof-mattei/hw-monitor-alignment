#![allow(
    clippy::as_conversions,
    reason = "Win32 FFI requires explicit integer casts"
)]
#![allow(
    clippy::multiple_unsafe_ops_per_block,
    reason = "Win32 interop batches several unsafe calls per block"
)]

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetWindowLongPtrW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, WS_MAXIMIZEBOX, WS_THICKFRAME,
};

/// Make `hwnd` a fixed-size dialog.
///
/// # Safety
/// `hwnd` must be a live top-level window handle owned by the calling thread.
pub unsafe fn make_fixed(hwnd: HWND) {
    // SAFETY: the caller guarantees `hwnd` is a live top-level window. Clearing the resize
    // styles and re-applying with `SWP_FRAMECHANGED` is the documented way to make a window
    // non-resizable.
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);

        // drop `WS_THICKFRAME` and `WS_MAXIMIZEBOX` so it can't be resized and shows no resize cursor on its edges.
        let fixed = style & !((WS_THICKFRAME | WS_MAXIMIZEBOX) as isize);

        // set style
        SetWindowLongPtrW(hwnd, GWL_STYLE, fixed);

        // apply style
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}
