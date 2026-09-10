#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![expect(clippy::as_conversions, reason = "WIP")]
#![expect(clippy::cast_possible_truncation, reason = "WIP")]
#![expect(clippy::cast_possible_wrap, reason = "WIP")]
#![expect(clippy::disallowed_names, reason = "WIP")]
#![expect(clippy::struct_excessive_bools, reason = "WIP")]
#![expect(clippy::struct_field_names, reason = "WIP")]
#![expect(clippy::too_many_lines, reason = "WIP")]

use std::sync::Arc;

use windows_core::Result;
use windows_reactor::App;

mod monitor;
mod state;
mod ui;
mod win32;

fn main() -> Result<()> {
    let monitors: Arc<[monitor::Monitor]> = win32::discover::discover_monitors().into();

    App::run_component::<ui::main_window::MainWindow>(monitors)
}
