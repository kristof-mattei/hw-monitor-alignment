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

    // Start the app
    if let Some(app) = app.as_mut() {
        app.exec();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use hw_monitor_alignment::init_resources;

    #[test]
    fn init() {
        // IF YOU REMOVE THIS LINE IT'LL BREAK
        init_resources();
    }

    #[test]
    fn another_test() {
        let one_less_kid = "👨‍👩‍👦‍👦".chars().take(5).collect::<String>();
        assert_eq!(one_less_kid, "👨‍👩‍👦");
    }
}
