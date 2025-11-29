use slint::ComponentHandle as _;

use crate::ui::MainWindow;

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

fn main() {
    let state = init();

    let main_window = state.main_window.clone_strong();

    main_window.run().unwrap();
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
