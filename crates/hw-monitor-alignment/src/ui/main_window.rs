use std::rc::Rc;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use windows::Win32::windef::HWND;
use windows::Win32::winuser::GetActiveWindow;
use windows_reactor::{
    Button, ChildrenControl as _, Component, ComponentContext, ComponentTimer,
    CompositionHostEvent, ContentControl as _, ContentDialog, ElementRef, Grid, GridChildExt as _,
    GridLength, HorizontalAlignment, LayoutControl as _, Orientation, StackPanel, Thickness,
    VerticalAlignment, View, ViewContext, WindowVisuals,
};

use super::info_panel::info_panel;
use crate::monitor::Monitor;
use crate::state::{AdjustSession, SharedSession};
use crate::ui::overview::overview_canvas;
use crate::win32::{discover, overlay, window_style};

const WINDOW_TITLE: &str = "HwMonitorAlignment";

const POLL_INTERVAL: Duration = Duration::from_millis(100);

fn on_resize(w: f64, h: f64) {
    // SAFETY: failure mode is returning an `HWND` where `.is_invalid()` returns `true`.
    let hwnd: HWND = unsafe { GetActiveWindow() };

    if !hwnd.0.is_null() {
        // SAFETY: `hwnd` is valid.
        unsafe {
            window_style::resize(hwnd, w, h).expect("Could not resize window");
        }
    }
}

#[derive(Clone)]
pub enum Message {
    ToggleAdjusting,
    ShowAbout(bool),
    Poll,
    Close,
}

/// A running alignment session; dropping it cancels the poll, queued `Poll` included.
struct Adjusting {
    session: SharedSession,
    timer: ComponentTimer,
    /// The `AdjustSession::version` the overview was last built from.
    last_version: u64,
}

pub struct MainWindow {
    /// The monitor list as discovered at startup.
    monitors: Arc<[Monitor]>,
    /// What the UI draws: `monitors`, or the working positions of the running session.
    display_monitors: Arc<[Monitor]>,
    adjusting: Option<Adjusting>,
    show_about: bool,
    /// Measures the info panel so the window can be resized to fit it.
    panel_probe: ElementRef<Grid>,
}

/// `set_timeout` is one-shot; each `Poll` schedules the next.
fn schedule_poll(context: &ComponentContext<MainWindow>) -> ComponentTimer {
    context
        .set_timeout(POLL_INTERVAL, Message::Poll)
        .expect("Could not schedule the session poll")
}

impl MainWindow {
    fn start_adjusting(&mut self, context: &ComponentContext<Self>) {
        let session: SharedSession =
            Rc::new(Mutex::new(AdjustSession::new(Arc::clone(&self.monitors))));

        overlay::create_overlays(&session);

        self.adjusting = Some(Adjusting {
            session,
            timer: schedule_poll(context),
            last_version: 0,
        });
    }

    fn stop_adjusting(&mut self) {
        let Some(Adjusting { session, .. }) = self.adjusting.take() else {
            return;
        };

        overlay::destroy_remaining_overlays(&session);

        // Re-read actual OS positions so the overview reflects the final state.
        // TODO assert fresh matches our state
        self.display_monitors = discover::discover_monitors().into();
    }

    fn poll(&mut self, context: &ComponentContext<Self>) {
        let Some(adjusting) = self.adjusting.as_mut() else {
            return;
        };

        let lock = adjusting
            .session
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        if lock.stop_requested {
            drop(lock);

            self.stop_adjusting();

            return;
        }

        if lock.version != adjusting.last_version {
            adjusting.last_version = lock.version;

            self.display_monitors = lock.monitors_with_working_y().into();
        }

        drop(lock);

        adjusting.timer = schedule_poll(context);
    }
}

impl Component for MainWindow {
    type Input = Arc<[Monitor]>;
    type Message = Message;

    fn create(input: &Self::Input, _context: &ComponentContext<Self>) -> Self {
        Self {
            monitors: Arc::clone(input),
            display_monitors: Arc::clone(input),
            adjusting: None,
            show_about: false,
            panel_probe: ElementRef::new(),
        }
    }

    fn update(&mut self, message: Self::Message, context: &ComponentContext<Self>) {
        match message {
            Message::ToggleAdjusting if self.adjusting.is_some() => self.stop_adjusting(),
            Message::ToggleAdjusting => self.start_adjusting(context),
            Message::ShowAbout(show) => self.show_about = show,
            Message::Poll => self.poll(context),
            Message::Close => {
                // We'll need to update this to revert the monitor alignment if we're
                // within the timer timeout.
                _ = context.window().request_close();
            },
        }
    }

    fn view(&self, _input: &Self::Input, context: &mut ViewContext<Self>) -> View {
        context.window_title(WINDOW_TITLE);
        context.window_visuals(WindowVisuals::new().client_size(570.0, 900.0));

        context.use_effect("fixed-window", (), || {
            // Once the UI mounts and the window is active, grab the HWND
            // SAFETY: failure mode is returning `NULL`
            let hwnd: HWND = unsafe { GetActiveWindow() };

            if !hwnd.0.is_null() {
                window_style::make_fixed(hwnd).expect("Failed to make window fixed");
            }

            None
        });

        let probe = self.panel_probe.clone();

        context.use_effect("panel-size", (), move || {
            let observation = probe.observe_composition_host(|event| match event {
                CompositionHostEvent::Ready { width, height, .. }
                | CompositionHostEvent::Metrics { width, height, .. } => on_resize(width, height),
            });

            Some(Box::new(move || drop(observation)))
        });

        let adjusting = self.adjusting.is_some();

        let button_bar = Grid::new()
            .columns([GridLength::STAR, GridLength::STAR])
            .margin(Thickness::uniform(16.0))
            .grid_row(2)
            .children((
                Button::new()
                    .on_click(context.message(Message::ShowAbout(true)))
                    .horizontal_alignment(HorizontalAlignment::Left)
                    .grid_column(0)
                    .content("About"),
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(8.0)
                    .horizontal_alignment(HorizontalAlignment::Right)
                    .grid_column(1)
                    .children((
                        Button::new()
                            .on_click(context.message(Message::ToggleAdjusting))
                            .content(if adjusting { "Stop" } else { "Adjust" }),
                        Button::new()
                            .on_click(context.message(Message::Close))
                            .content("Close"),
                    )),
            ));

        let layout = Grid::new()
            .rows([
                GridLength::Pixel(200_f64),
                GridLength::STAR,
                GridLength::Auto,
            ])
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch)
            .children((
                overview_canvas(&self.display_monitors)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .margin(Thickness::uniform(16.0))
                    .grid_row(0),
                // `info_panel` returns a `View`, which cannot take `grid_row`
                Grid::new()
                    .grid_row(1)
                    .children([info_panel(&self.display_monitors, &self.panel_probe)]),
                button_bar,
            ));

        Grid::new()
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch)
            .children((layout, about_dialog(self.show_about, context)))
    }
}

fn about_dialog(show_about: bool, context: &ViewContext<MainWindow>) -> View {
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

    ContentDialog::new()
        .title("About HwMonitorAlignment")
        .close_button_text("OK")
        .is_open(show_about)
        .on_closed(context.callback(|_result| Message::ShowAbout(false)))
        .content(about_text)
}
