#[allow(clippy::unnecessary_box_returns)]
pub mod cxxqt_object;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;
    // Create the application and engine
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // Load the QML path into the engine
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/hw_monitor_alignment/qml/main.qml"));
    }

    // Start the app
    if let Some(app) = app.as_mut() {
        app.exec();
    }

    Ok(())
}
