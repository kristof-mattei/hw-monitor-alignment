use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        // Link Qt's Network library
        // - Qt Core is always linked
        // - Qt Gui is linked by enabling the qt_gui Cargo feature (default).
        // - Qt Qml is linked by enabling the qt_qml Cargo feature (default).
        // - Qt Qml requires linking Qt Network on macOS
        // if mac
        .qt_module("Network")
        .qml_module(QmlModule {
            uri: "hw_monitor_alignment",
            rust_files: &["src/cxxqt_object.rs"],
            qml_files: &["qml/main.qml"],
            // qrc_files: &["qml/qml.qrc"],
            ..Default::default()
        })
        // .file("src/cxxqt_object.rs")
        .qrc("qml/qml.qrc")
        // .with_opts(cxx_qt_lib_headers::build_opts())
        .build();
}
