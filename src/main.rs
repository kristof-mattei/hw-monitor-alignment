use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use hw_monitor_alignment::init_resources;

fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;

    // IF YOU REMOVE THIS LINE IT'LL BREAK
    init_resources();

    // Create the application and engine
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // Load the QML path into the engine
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/hw_monitor_alignment/qml/main.qml"));
    }

    if let Some(engine) = engine.as_mut() {
        // doesn't seem to work?
        // Listen to a signal from the QML Engine
        engine
            .as_qqmlengine()
            .on_quit(|_| {
                println!("QML Quit!");
            })
            .release();
    }

    // Start the app
    if let Some(app) = app.as_mut() {
        app.exec();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[expect(
        unused,
        reason = "To ensure QT stuff gets included when compiling for tests"
    )]
    use hw_monitor_alignment::init_resources;
}
