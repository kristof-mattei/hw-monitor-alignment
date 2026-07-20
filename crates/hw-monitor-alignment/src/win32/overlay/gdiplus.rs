//! Minimal flat GDI+ bindings.

#![expect(
    non_snake_case,
    non_upper_case_globals,
    reason = "names mirror the flat GDI+ API"
)]

use windows::Win32::windef::HDC;

/// GDI+ `GpStatus`: `Status(0)` (`Ok`) means success.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status(pub i32);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unit(pub i32);

pub const UnitPixel: Unit = Unit(2);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmoothingMode(pub i32);

pub const SmoothingModeNone: SmoothingMode = SmoothingMode(3);
pub const SmoothingModeAntiAlias: SmoothingMode = SmoothingMode(4);

/// Opaque GDI+ graphics context, only ever used behind a raw pointer.
#[repr(C)]
pub struct GpGraphics(pub u8);

/// Opaque GDI+ pen, only ever used behind a raw pointer.
#[repr(C)]
pub struct GpPen(pub u8);

#[repr(C)]
pub struct GdiplusStartupInput {
    pub GdiplusVersion: u32,
    pub DebugEventCallback: isize,
    pub SuppressBackgroundThread: windows_core::BOOL,
    pub SuppressExternalCodecs: windows_core::BOOL,
}

#[repr(C)]
pub struct GdiplusStartupOutput {
    pub NotificationHook: isize,
    pub NotificationUnhook: isize,
}

pub unsafe fn GdiplusStartup(
    token: *mut usize,
    input: *const GdiplusStartupInput,
    output: *mut GdiplusStartupOutput,
) -> Status {
    windows_core::link!("gdiplus.dll" "system" fn GdiplusStartup(token : *mut usize, input : *const GdiplusStartupInput, output : *mut GdiplusStartupOutput) -> Status);

    // SAFETY: forwarded
    unsafe { GdiplusStartup(token, input, output) }
}

pub unsafe fn GdipCreateFromHDC(hdc: HDC, graphics: *mut *mut GpGraphics) -> Status {
    windows_core::link!("gdiplus.dll" "system" fn GdipCreateFromHDC(hdc : HDC, graphics : *mut *mut GpGraphics) -> Status);

    // SAFETY: forwarded
    unsafe { GdipCreateFromHDC(hdc, graphics) }
}

pub unsafe fn GdipSetSmoothingMode(
    graphics: *mut GpGraphics,
    smoothingmode: SmoothingMode,
) -> Status {
    windows_core::link!("gdiplus.dll" "system" fn GdipSetSmoothingMode(graphics : *mut GpGraphics, smoothingmode : SmoothingMode) -> Status);

    // SAFETY: forwarded
    unsafe { GdipSetSmoothingMode(graphics, smoothingmode) }
}

pub unsafe fn GdipCreatePen1(color: u32, width: f32, unit: Unit, pen: *mut *mut GpPen) -> Status {
    windows_core::link!("gdiplus.dll" "system" fn GdipCreatePen1(color : u32, width : f32, unit : Unit, pen : *mut *mut GpPen) -> Status);

    // SAFETY: forwarded
    unsafe { GdipCreatePen1(color, width, unit, pen) }
}

pub unsafe fn GdipDrawLine(
    graphics: *mut GpGraphics,
    pen: *mut GpPen,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> Status {
    windows_core::link!("gdiplus.dll" "system" fn GdipDrawLine(graphics : *mut GpGraphics, pen : *mut GpPen, x1 : f32, y1 : f32, x2 : f32, y2 : f32) -> Status);

    // SAFETY: forwarded
    unsafe { GdipDrawLine(graphics, pen, x1, y1, x2, y2) }
}

pub unsafe fn GdipDeletePen(pen: *mut GpPen) -> Status {
    windows_core::link!("gdiplus.dll" "system" fn GdipDeletePen(pen : *mut GpPen) -> Status);

    // SAFETY: forwarded
    unsafe { GdipDeletePen(pen) }
}

pub unsafe fn GdipDeleteGraphics(graphics: *mut GpGraphics) -> Status {
    windows_core::link!("gdiplus.dll" "system" fn GdipDeleteGraphics(graphics : *mut GpGraphics) -> Status);

    // SAFETY: forwarded
    unsafe { GdipDeleteGraphics(graphics) }
}
