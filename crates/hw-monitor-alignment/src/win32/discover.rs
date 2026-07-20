use windows::Win32::wingdi::{
    DEVMODEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_MIRRORING_DRIVER,
    DISPLAY_DEVICE_PRIMARY_DEVICE, DISPLAY_DEVICEW,
};
use windows::Win32::winuser::{
    EDD_GET_DEVICE_INTERFACE_NAME, ENUM_REGISTRY_SETTINGS, EnumDisplayDevicesW,
    EnumDisplaySettingsW,
};
use windows_core::PCWSTR;

use crate::monitor::{Monitor, Orientation};
use crate::win32::friendly;

/// Discovers all the monitors on a system.
///
/// Return value is sorted by the monitor's (x, y).
pub fn discover_monitors() -> Vec<Monitor> {
    // Friendly names come from a separate CCD-API pass, keyed by device path.
    let friendly_names = friendly::discover_friendly_names();

    let mut monitors = Vec::new();
    // display adaptor id
    let mut i_dev: u32 = 0;

    loop {
        let mut dd = DISPLAY_DEVICEW {
            cb: u32::try_from(size_of::<DISPLAY_DEVICEW>()).unwrap(),
            ..DISPLAY_DEVICEW::default()
        };

        // SAFETY: dd is initialised above and lives for the call duration.
        let found_display_adaptor =
            unsafe { EnumDisplayDevicesW(PCWSTR::null(), i_dev, &raw mut dd, 0) }.as_bool();

        if !found_display_adaptor {
            break;
        }

        // monitor id, per display adaptor
        let mut device_monitor: u32 = 0;

        loop {
            let mut ddm = DISPLAY_DEVICEW {
                cb: u32::try_from(size_of::<DISPLAY_DEVICEW>()).unwrap(),
                ..DISPLAY_DEVICEW::default()
            };

            // SAFETY: API call, `ddm` is correctly initialized, `dd.DeviceName` comes from another API call
            let found_monitor = unsafe {
                EnumDisplayDevicesW(
                    PCWSTR::from_raw(dd.DeviceName.as_ptr()),
                    device_monitor,
                    &raw mut ddm,
                    EDD_GET_DEVICE_INTERFACE_NAME,
                )
            }
            .as_bool();

            if !found_monitor {
                break;
            }

            let is_attached = (ddm.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) != 0;
            let is_mirroring = (ddm.StateFlags & DISPLAY_DEVICE_MIRRORING_DRIVER) != 0;

            if is_attached && !is_mirroring {
                let mut devmode = DEVMODEW {
                    dmSize: u16::try_from(size_of::<DEVMODEW>()).unwrap(),
                    ..DEVMODEW::default()
                };

                // SAFETY: `dd.DeviceName` comes from another API call, `devmode` is initialized.
                let ok = unsafe {
                    EnumDisplaySettingsW(
                        PCWSTR::from_raw(dd.DeviceName.as_ptr()),
                        ENUM_REGISTRY_SETTINGS,
                        &raw mut devmode,
                    )
                }
                .as_bool();

                if ok {
                    // SAFETY: reading nested anonymous union.
                    let display = unsafe { devmode.Anonymous.Anonymous2 };
                    let pos = display.dmPosition;

                    let device_id = String::from_utf16_lossy(&ddm.DeviceID);

                    // fetch friendly name
                    let friendly_monitor_name = friendly_names
                        .get(&device_id)
                        .map_or("", |s| &**s)
                        .to_owned();

                    monitors.push(Monitor {
                        device_name: String::from_utf16_lossy(&dd.DeviceName).into(),
                        monitor_name: String::from_utf16_lossy(&ddm.DeviceString).into(),
                        friendly_monitor_name: friendly_monitor_name.into(),
                        display_adapter: String::from_utf16_lossy(&dd.DeviceString).into(),
                        width: devmode.dmPelsWidth,
                        height: devmode.dmPelsHeight,
                        x: pos.x,
                        y: pos.y,
                        orientation: Orientation::from_dmdo(display.dmDisplayOrientation),
                        primary: (dd.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0,
                    });
                }
            }

            device_monitor += 1;
        }

        i_dev += 1;
    }

    monitors.sort_by_key(|m| (m.x, m.y));

    monitors
}
