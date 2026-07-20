use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use windows::Win32::windef::HWND;
use windows::Win32::winuser::{GetActiveWindow, PostQuitMessage};
use windows_reactor::{
    ContentDialog, DispatcherTimer, Element, ElementExt as _, GridLength, HorizontalAlignment,
    RenderCx, SetState, Thickness, VerticalAlignment, button, grid, hstack,
};

use super::info_panel::info_panel;
use crate::monitor::Monitor;
use crate::state::{AdjustSession, SharedSession};
use crate::ui::overview::overview_canvas;
use crate::win32::{discover, overlay, window_style};

fn on_resize(w: f64, h: f64) {
    // Once the UI mounts and the window is active, grab the HWND
    // SAFETY: failure mode is returning an `HWND` where `.is_invalid()` returns `true`.
    let hwnd: HWND = unsafe { GetActiveWindow() };

    if !hwnd.0.is_null() {
        // SAFETY: `hwnd` is valid.
        unsafe {
            window_style::resize(hwnd, w, h).expect("Could not resize window");
        }
    }
}

pub fn render(cx: &mut RenderCx, monitors: &Arc<[Monitor]>) -> impl Into<Element> {
    let (adjusting, set_adjusting) = cx.use_state(false);

    let (display_monitors, set_display_monitors) = cx.use_state(Arc::clone(monitors));
    let (show_about, set_show_about) = cx.use_state(false);

    cx.use_effect((), || {
        // Once the UI mounts and the window is active, grab the HWND
        // SAFETY: failure mode is returning `NULL`
        let hwnd: HWND = unsafe { GetActiveWindow() };

        if !hwnd.0.is_null() {
            window_style::make_fixed(hwnd).expect("Failed to make window fixed");
        }
    });

    {
        let monitors = Arc::clone(monitors);
        let set_adjusting = SetState::clone(&set_adjusting);

        cx.use_effect_with_cleanup(adjusting, move || {
            if !adjusting {
                return None::<Box<dyn FnOnce()>>;
            }

            let session: SharedSession = Rc::new(std::sync::Mutex::new(AdjustSession::new(
                Arc::clone(&monitors),
            )));

            overlay::create_overlays(&session);

            let last_version = std::cell::Cell::new(0_u64);

            let timer = {
                let session = Rc::clone(&session);
                let set_display_monitors = SetState::clone(&set_display_monitors);

                DispatcherTimer::new(Duration::from_millis(100), move || {
                    let lock = session
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);

                    if lock.stop_requested {
                        drop(lock);

                        set_adjusting.call(false);

                        return;
                    }

                    if lock.version != last_version.get() {
                        last_version.set(lock.version);

                        let updated = lock.monitors_with_working_y().into();

                        drop(lock);

                        set_display_monitors.call(updated);
                    }
                })
                .ok()
            };

            let cleanp: Box<dyn FnOnce()> = {
                let session = Rc::clone(&session);
                let set_display = SetState::clone(&set_display_monitors);

                Box::new(move || {
                    drop(timer);

                    overlay::destroy_remaining_overlays(&session);

                    // Re-read actual OS positions so the overview reflects the final state.
                    let fresh = discover::discover_monitors().into();

                    // TODO assert fresh matches our state

                    set_display.call(fresh);
                })
            };

            Some(cleanp)
        });
    }

    let button_bar = {
        grid((
            {
                let set_show_about = SetState::clone(&set_show_about);

                button("About")
                    .on_click(move || set_show_about.call(true))
                    .horizontal_alignment(HorizontalAlignment::Left)
                    .grid_column(0)
            },
            hstack([
                button(if adjusting { "Stop" } else { "Adjust" })
                    .on_click(move || set_adjusting.call(!adjusting)),
                button("Close").on_click(close_app),
            ])
            .spacing(8.0)
            .horizontal_alignment(HorizontalAlignment::Right)
            .grid_column(1),
        ))
        .columns([GridLength::STAR, GridLength::STAR])
        .margin(Thickness::uniform(16.0))
    };

    let layout = grid((
        overview_canvas(&display_monitors)
            .horizontal_alignment(HorizontalAlignment::Center)
            .margin(Thickness::uniform(16.0))
            .grid_row(0),
        info_panel(&display_monitors, on_resize)
            .horizontal_alignment(HorizontalAlignment::Stretch)
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

    let children: Vec<Element> = vec![layout.into(), content_dialog(show_about, set_show_about)];

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
