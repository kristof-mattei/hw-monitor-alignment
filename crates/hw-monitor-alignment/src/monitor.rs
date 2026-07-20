use windows::Win32::wingdi::{DMDO_90, DMDO_180, DMDO_270};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    Landscape,
    Portrait,
    FlippedLandscape,
    FlippedPortrait,
}

impl Orientation {
    /// Convert a Win32 `dmDisplayOrientation` (`DMDO_DEFAULT`/`DMDO_90`/`DMDO_180`/`DMDO_270`).
    pub fn from_dmdo(value: u32) -> Self {
        match value {
            DMDO_90 => Self::Portrait,
            DMDO_180 => Self::FlippedLandscape,
            DMDO_270 => Self::FlippedPortrait,
            // `DMDO_DEFAULT`
            _ => Self::Landscape,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Landscape => "Landscape",
            Self::Portrait => "Portrait",
            Self::FlippedLandscape => "Landscape (flipped)",
            Self::FlippedPortrait => "Portrait (flipped)",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Monitor {
    pub device_name: Box<str>,
    pub monitor_name: Box<str>,
    pub friendly_monitor_name: Box<str>,
    pub display_adapter: Box<str>,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub orientation: Orientation,
    pub primary: bool,
}
