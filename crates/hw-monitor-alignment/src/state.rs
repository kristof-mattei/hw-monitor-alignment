use std::rc::Rc;
use std::sync::{Arc, Mutex};

use windows::windef::HWND;

use crate::monitor::Monitor;

#[derive(Debug)]
pub struct OverlaySettings {
    pub show_diagonal_lines: bool,
    pub show_horizontal_lines: bool,
    #[expect(
        dead_code,
        reason = "only read by the info box, which is temporarily removed"
    )]
    pub show_info_box: bool,
    pub show_cursor_position: bool,
    pub antialiasing: bool,
    pub line_spacing: i32,
    pub line_thickness: i32,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            show_diagonal_lines: false,
            show_horizontal_lines: true,
            show_info_box: true,
            show_cursor_position: false,
            antialiasing: false,
            line_spacing: 100,
            line_thickness: 3,
        }
    }
}

pub struct AdjustSession {
    pub monitors: Arc<[Monitor]>,
    pub working_y: Vec<i32>,
    pub version: u64,
    pub stop_requested: bool,
    /// Every window created for the session, overlays and the owned box windows.
    // Used for teardown and repaint invalidation.
    pub hwnd_list: Vec<HWND>,
    /// Just the info-box windows, so the "Show info box" toggle can hide/show them.
    #[expect(
        dead_code,
        reason = "only read by the info box, which is temporarily removed"
    )]
    pub info_box_hwnds: Vec<HWND>,
    pub settings: OverlaySettings,
}

impl AdjustSession {
    pub fn new(monitors: Arc<[Monitor]>) -> Self {
        let working_y = monitors.iter().map(|m| m.y).collect();
        Self {
            monitors,
            working_y,
            version: 0,
            stop_requested: false,
            hwnd_list: Vec::new(),
            info_box_hwnds: Vec::new(),
            settings: OverlaySettings::default(),
        }
    }

    /// Reset every working position back to its pristine (discovered) value.
    #[expect(
        dead_code,
        reason = "used by the Reset/Revert flow of the control box, which is temporarily removed"
    )]
    pub fn rollback(&mut self) {
        for (working, monitor) in self.working_y.iter_mut().zip(self.monitors.iter()) {
            *working = monitor.y;
        }
    }

    /// The monitor list with current working positions applied, for the UI overview.
    pub fn monitors_with_working_y(&self) -> Vec<Monitor> {
        self.monitors
            .iter()
            .zip(self.working_y.iter())
            .map(|(m, &y)| {
                let mut m = m.clone();

                m.y = y;

                m
            })
            .collect()
    }
}

pub type SharedSession = Rc<Mutex<AdjustSession>>;
