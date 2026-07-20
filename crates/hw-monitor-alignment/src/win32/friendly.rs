#![expect(
    clippy::as_conversions,
    reason = "Win32 FFI requires explicit integer casts"
)]

//! Friendly monitor names via the Win32 Connecting-and-Configuring-Displays (CCD) API.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt as _;

use hashbrown::HashMap;
use windows::Win32::winerror::ERROR_SUCCESS;
use windows::Win32::wingdi::{
    DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_DEVICE_INFO_HEADER,
    DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME,
    QDC_ONLY_ACTIVE_PATHS,
};
use windows::Win32::winuser::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
};
use windows_core::WIN32_ERROR;

fn trim(mut wide: &[u16]) -> &[u16] {
    while wide.last() == Some(&0) {
        wide = &wide[..wide.len() - 1];
    }

    wide
}

fn wstr_to_string(wide: &[u16]) -> String {
    let wide = trim(wide);

    let r = OsString::from_wide(wide);

    r.to_string_lossy().into_owned()
}

/// Mapping of `(monitorDevicePath, monitorFriendlyDeviceName)` pairs.
pub fn discover_friendly_names() -> HashMap<String, String> {
    let mut num_path_array_elements: u32 = 0;
    let mut num_mode_info_array_elements: u32 = 0;

    // SAFETY: Win32 API call
    let rc = unsafe {
        GetDisplayConfigBufferSizes(
            QDC_ONLY_ACTIVE_PATHS,
            &raw mut num_path_array_elements,
            &raw mut num_mode_info_array_elements,
        )
    };

    if rc.cast_unsigned() != ERROR_SUCCESS || num_path_array_elements == 0 {
        return HashMap::new();
    }

    let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> =
        vec![DISPLAYCONFIG_PATH_INFO::default(); num_path_array_elements as usize];

    let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> =
        vec![DISPLAYCONFIG_MODE_INFO::default(); num_mode_info_array_elements as usize];

    // SAFETY: arrays are sized per `GetDisplayConfigBufferSizes`.
    let rc = unsafe {
        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &raw mut num_path_array_elements,
            paths.as_mut_ptr(),
            &raw mut num_mode_info_array_elements,
            modes.as_mut_ptr(),
            std::ptr::null_mut(),
        )
    };

    if rc.cast_unsigned() != ERROR_SUCCESS {
        return HashMap::new();
    }

    let mut mapping: HashMap<String, String> = HashMap::new();

    for path in paths.iter().take(num_path_array_elements as usize) {
        let mut target: DISPLAYCONFIG_TARGET_DEVICE_NAME = DISPLAYCONFIG_TARGET_DEVICE_NAME {
            header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                size: u32::try_from(size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>()).unwrap(),
                adapterId: path.targetInfo.adapterId,
                id: path.targetInfo.id,
            },
            ..Default::default()
        };

        // SAFETY: target.header is a valid `DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME` packet sized via header.size;
        // the call fills the surrounding struct in place.
        let rc = unsafe { DisplayConfigGetDeviceInfo(&raw mut target.header) };

        if WIN32_ERROR(rc.cast_unsigned()).is_err() {
            continue;
        }

        let device_path = wstr_to_string(&target.monitorDevicePath);
        let friendly = wstr_to_string(&target.monitorFriendlyDeviceName);

        if !device_path.is_empty() {
            mapping.insert(device_path, friendly);
        }
    }

    mapping
}
