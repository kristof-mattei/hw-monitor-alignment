#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![expect(clippy::as_conversions, reason = "WIP")]
#![expect(clippy::cast_possible_truncation, reason = "WIP")]
#![expect(clippy::cast_possible_wrap, reason = "WIP")]
#![expect(clippy::disallowed_names, reason = "WIP")]
#![expect(clippy::struct_excessive_bools, reason = "WIP")]
#![expect(clippy::struct_field_names, reason = "WIP")]
#![expect(clippy::too_many_lines, reason = "WIP")]

use std::sync::Arc;

use windows_reactor::{App, Result, bootstrap};

#[expect(
    non_snake_case,
    clippy::absolute_paths,
    clippy::borrow_as_ptr,
    clippy::multiple_unsafe_ops_per_block,
    clippy::partial_pub_fields,
    clippy::ptr_as_ptr,
    clippy::transmute_ptr_to_ptr,
    clippy::undocumented_unsafe_blocks,
    reason = "Generated"
)]
mod bindings;
mod monitor;
mod state;
mod ui;
mod win32;

const WINDOW_TITLE: &str = "HwMonitorAlignment";

fn main() -> Result<()> {
    // ensure we have the WinUI3 package.
    bootstrap()?;

    let monitors: Arc<[monitor::Monitor]> = win32::discover::discover_monitors().into();

    App::new()
        .title(WINDOW_TITLE)
        .inner_size(570.0, 900.0)
        .render(move |cx| ui::main_window::render(cx, &monitors).into())
}
