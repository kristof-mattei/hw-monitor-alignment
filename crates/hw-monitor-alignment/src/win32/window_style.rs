use windows::Win32::Foundation::{HWND, SetLastError};
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetWindowLongPtrW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, WINDOW_STYLE, WS_MAXIMIZEBOX, WS_THICKFRAME,
};
use windows_core::WIN32_ERROR;

/// Make `hwnd` a fixed-size dialog.
///
/// Why this is not unsafe: If `hwnd` is invalid the function remains safe.
pub fn make_fixed(hwnd: HWND) -> Result<(), windows_reactor::Error> {
    // SAFETY: Clear the error state before querying so we don't read stale errors.
    unsafe {
        SetLastError(WIN32_ERROR(0));
    }

    // SAFETY: Even an invalid `hwnd` won't make this fail
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) };

    if style == 0 {
        // style == 0 doesn't mean there is an error, we need to explicitly check for that.
        WIN32_ERROR::from_thread().ok()?;
    }

    // drop `WS_THICKFRAME` and `WS_MAXIMIZEBOX` so it can't be resized and shows no resize cursor on its edges.
    let fixed = WINDOW_STYLE(
        style
            .try_into()
            .expect("GetWindowLongPtrW(hwnd, GWL_STYLE) returns WINDOW_STYLE"),
    ) & !(WS_THICKFRAME | WS_MAXIMIZEBOX);

    // SAFETY: unset the errors so we can properly inspect return values
    unsafe {
        SetLastError(WIN32_ERROR(0));
    }

    // SAFETY: `hwnd` is valid, so is the `fixed` style
    let result = unsafe {
        // set style
        SetWindowLongPtrW(hwnd, GWL_STYLE, fixed.0 as isize)
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
    }?;

    Ok(())
}
