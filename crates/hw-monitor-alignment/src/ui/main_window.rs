use std::sync::Arc;

use windows_reactor::{
    ContentDialog, Element, ElementExt as _, GridLength, HorizontalAlignment, RenderCx, SetState,
    Thickness, VerticalAlignment, button, grid, hstack,
};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::PostQuitMessage;

use super::info_panel::info_panel;
use crate::monitor::Monitor;
use crate::ui::overview::overview_canvas;
use crate::win32::window_style;

pub fn render(cx: &mut RenderCx, monitors: &Arc<[Monitor]>) -> impl Into<Element> {
    let (adjusting, set_adjusting) = cx.use_state(false);
    // TODO this shouldn't clone the monitors
    let (display_monitors, _set_display_monitors) = cx.use_state(monitors.to_vec());
    let (show_about, set_show_about) = cx.use_state(false);

    cx.use_effect((), || {
        // Once the UI mounts and the window is active, grab the HWND
        // SAFETY: failure mode is returning `NULL`
        let hwnd: HWND = unsafe { GetActiveWindow() };

        if !hwnd.is_null() {
            window_style::make_fixed(hwnd).expect("Failed to make window fixed");
        }
    });

    let set_adjust_button = set_adjusting.clone();
    let set_about_button = set_show_about.clone();
    let set_about_close = set_show_about.clone();

    let button_bar = grid((
        button("About")
            .on_click(move || set_about_button.call(true))
            .horizontal_alignment(HorizontalAlignment::Left)
            .grid_column(0),
        hstack([
            button(if adjusting { "Stop" } else { "Adjust" })
                .on_click(move || set_adjust_button.call(!adjusting)),
            button("Close").on_click(close_app),
        ])
        .spacing(8.0)
        .horizontal_alignment(HorizontalAlignment::Right)
        .grid_column(1),
    ))
    .columns([GridLength::STAR, GridLength::STAR])
    .margin(Thickness::uniform(16.0));

    let layout = grid((
        overview_canvas(&display_monitors)
            .horizontal_alignment(HorizontalAlignment::Center)
            .margin(Thickness::uniform(16.0))
            .grid_row(0),
        info_panel(&display_monitors)
            .horizontal_alignment(HorizontalAlignment::Left)
            .vertical_alignment(VerticalAlignment::Stretch)
            .grid_row(1),
        button_bar.grid_row(2),
    ))
    .rows([
        GridLength::Pixel(200_f64),
        GridLength::STAR,
        GridLength::Auto,
    ])
    .horizontal_alignment(HorizontalAlignment::Stretch)
    .vertical_alignment(VerticalAlignment::Stretch);

    let children: Vec<Element> = vec![layout.into(), content_dialog(show_about, set_about_close)];

    grid(children)
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .vertical_alignment(VerticalAlignment::Stretch)
}

fn content_dialog(show_about: bool, set_about_close: SetState<bool>) -> Element {
    let about_text = format!(
        "{} {}\n\n\
         {}\n\n\
         Repository: {}\n\n\
         Rust / WinUI3 port of the original Python application.\n\n\
         Third-party components:\n\
         - windows-reactor (WinUI3 bindings)\n\
         - windows-sys (Win32 bindings)\n\n\
         Usage: press Adjust to open alignment overlays on each monitor; click the screen \
         you want to move, then use Up/Down (or Page Up/Page Down) to move it, click Apply, \
         then Keep or Revert within 15 seconds.",
        "HwMonitorAlignment",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_REPOSITORY"),
    );

    ContentDialog::new("About HwMonitorAlignment")
        .content(about_text)
        .close_button_text("OK")
        .is_open(show_about)
        .on_closed(move |_| set_about_close.call(false))
        .into()
}

fn close_app() {
    // SAFETY: instruct the UI broker to shut down.
    // We'll need to update this to revert the monitor alignment if we're within the timer timeout.
    unsafe {
        PostQuitMessage(0);
    }
}
