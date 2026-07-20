use windows::Win32::errhandlingapi::SetLastError;
use windows::Win32::windef::{HWND, RECT};
use windows::Win32::winerror::E_INVALIDARG;
use windows::Win32::winuser::{
    AdjustWindowRectExForDpi, GWL_EXSTYLE, GWL_STYLE, GetDpiForSystem, GetDpiForWindow,
    GetWindowLongPtrW, GetWindowRect, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, USER_DEFAULT_SCREEN_DPI, WS_MAXIMIZEBOX,
    WS_THICKFRAME,
};
use windows_core::WIN32_ERROR;

/// Make `hwnd` a fixed-size dialog.
///
/// Why this is not unsafe: If `hwnd` is invalid the function remains safe.
pub fn make_fixed(hwnd: HWND) -> Result<(), windows_reactor::Error> {
    // SAFETY: Clear the error state before querying so we don't read stale errors.
    unsafe {
        SetLastError(0);
    }

    // SAFETY: Even an invalid `hwnd` won't make this fail
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) };

    if style == 0 {
        // style == 0 doesn't mean there is an error, we need to explicitly check for that.
        WIN32_ERROR::from_thread().ok()?;
    }

    // drop `WS_THICKFRAME` and `WS_MAXIMIZEBOX` so it can't be resized and shows no resize cursor on its edges.
    let fixed: u32 = u32::try_from(style)
        .expect("GetWindowLongPtrW(hwnd, GWL_STYLE) returns a window style")
        & !(WS_THICKFRAME | WS_MAXIMIZEBOX);

    // SAFETY: unset the errors so we can properly inspect return values
    unsafe {
        SetLastError(0);
    }

    // SAFETY: `hwnd` is valid, so is the `fixed` style
    let result = unsafe {
        // set style
        SetWindowLongPtrW(hwnd, GWL_STYLE, fixed as isize)
    };

    // A return of 0 is the only indicator that we need to check the error state.
    if result == 0 {
        WIN32_ERROR::from_thread().ok()?;
    }

    // SAFETY: documented way to prevent resizing
    unsafe {
        SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        )
    }
    .ok()?;

    Ok(())
}

fn get_dpi(hwnd: HWND) -> u32 {
    // SAFETY: the caller guarantees `hwnd` is a valid `HWND`
    let window_dpi = unsafe { GetDpiForWindow(hwnd) };

    if window_dpi != 0 {
        return window_dpi;
    }

    // SAFETY: Win32 API call
    let system_dpi = unsafe { GetDpiForSystem() };

    if system_dpi != 0 {
        return system_dpi;
    }

    USER_DEFAULT_SCREEN_DPI
}

/// Resize `hwnd` to the given client **width** `w` (logical/DIP units), keeping
/// the window's current height and position.
///
/// # Safety
/// `hwnd` must be a live top-level window handle owned by the calling thread.
pub unsafe fn resize(hwnd: HWND, w: f64, _h: f64) -> Result<(), windows_core::Error> {
    if w <= 0.0 || w.is_nan() {
        return Err(windows_core::Error::new(
            E_INVALIDARG,
            "Width must be a finite, positive number.",
        ));
    }

    let dpi = get_dpi(hwnd);

    // determine the scale, so that we can take device-independent-pixel count, multiply it by the scale, and get the real pixel count
    let scale = f64::from(dpi) / 96.0;

    let wanted_internal_width = (w * scale).round() as i32;

    // Get the bounds of the WHOLE window, including non-client area
    let current_rect = {
        let mut current = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        // SAFETY: Win32 API call
        unsafe { GetWindowRect(hwnd, &raw mut current) }.ok()?;

        current
    };

    let current_width = current_rect.right - current_rect.left;
    let current_height = current_rect.bottom - current_rect.top;

    // Grow the desired client width into a full window width, honoring the
    // window's actual styles and this monitor's DPI (left/right borders).
    // We're only interested in the width, as we don't want to change the height.

    let new_width = {
        #[expect(
            clippy::cast_sign_loss,
            reason = "GetWindowLongPtrW returns GWL_STYLE in the lower 32-bits of the return value"
        )]
        // SAFETY: Win32 API call
        let window_style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 };

        #[expect(
            clippy::cast_sign_loss,
            reason = "GetWindowLongPtrW returns GWL_EXSTYLE in the lower 32-bits of the return value"
        )]
        // SAFETY: Win32 API call
        let window_ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 };

        let mut adjusted = RECT {
            left: 0,
            top: 0,
            right: wanted_internal_width,
            bottom: 0,
        };

        // add the non-client-area to our new width
        // SAFETY: Win32 API call
        unsafe {
            AdjustWindowRectExForDpi(&raw mut adjusted, window_style, false, window_ex_style, dpi)
        }
        .ok()?;

        adjusted.right - adjusted.left
    };

    // Calculate the difference in width, and shift the window half of that to the left
    let delta_width = new_width - current_width;
    let new_x = current_rect.left - (delta_width / 2);

    // We aren't changing height
    let new_y = current_rect.top;

    // nothing we can do if this fails
    // SAFETY: Win32 API call
    unsafe {
        SetWindowPos(
            hwnd,
            None,
            new_x,
            new_y,
            new_width,
            current_height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        )
    }
    .ok()
}
