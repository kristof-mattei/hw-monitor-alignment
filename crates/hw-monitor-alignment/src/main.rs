use color_eyre::config::HookBuilder;
use color_eyre::eyre;
use slint::ComponentHandle as _;

use crate::ui::MainWindow;
mod build_env;
use build_env::get_build_env;

pub mod ui {
    #![allow(
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::restriction,
        let_underscore_drop,
        reason = "slint has lots of rust violations"
    )]
    slint::include_modules!();
}

fn print_header() {
    const NAME: &str = env!("CARGO_PKG_NAME");
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let build_env = get_build_env();

    println!(
        "{} v{} - built for {} ({})",
        NAME,
        VERSION,
        build_env.get_target(),
        build_env.get_target_cpu().unwrap_or("base cpu variant"),
    );
}

fn main() -> Result<(), eyre::Report> {
    HookBuilder::default()
        .capture_span_trace_by_default(true)
        .install()?;

    print_header();

    let state = init();

    let main_window = state.main_window.clone_strong();

    main_window.run().map_err(Into::into)
}

fn init() -> State {
    let main_window = MainWindow::new().unwrap();

    State {
        main_window,
        // todo_model,
    }
}

pub struct State {
    pub main_window: MainWindow,
    // pub todo_model: Rc<slint::VecModel<TodoItem>>,
}
