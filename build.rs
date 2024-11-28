use std::{env, fs::OpenOptions};
use std::{io::Write, path::PathBuf};

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    let modules = [QmlModule {
        uri: "hw_monitor_alignment",
        version_major: 1,
        version_minor: 0,
        rust_files: &["src/cxxqt_object.rs"],
        qml_files: &["qml/main.qml"],
        ..Default::default()
    }];

    let mut builder = CxxQtBuilder::new().cc_builder(|cc| {
        let start = r#"
#include <QtPlugin>

void _init_qt_resources() {
    std::fprintf(stderr, "Initializing plugins...\n");
            "#;

        let end = r#"
    std::fprintf(stderr, "Done initializing plugins...\n");
}

extern "C" {
    void init_qt_resources() {
        _init_qt_resources();
    }
}
            "#;

        let mut cpp_directory = PathBuf::from(env::var("OUT_DIR").unwrap());
        cpp_directory.push("cxx-qt-gen/src");

        std::fs::create_dir_all(&cpp_directory).expect("Couldn't create dir");

        let mut cpp_file = cpp_directory.clone();
        cpp_file.push("init_hack.cpp");

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&cpp_file)
            .expect("Couldn't create file");

        writeln!(file, "{}", start).expect("Failed to write start");

        for m in &modules {
            writeln!(file, "    Q_IMPORT_PLUGIN({}_plugin)", m.uri).expect("Failed to write piece");
        }

        writeln!(file, "{}", end).expect("Failed to write end");

        cc.file(cpp_file);
    });

    for m in modules {
        builder = builder.qml_module(m);
    }
    // if mac
    // .qt_module("Network")

    builder.build();
}
