#[expect(clippy::unnecessary_box_returns)]
pub mod cxxqt_object;

pub fn init_resources() {
    #[link(name = "hw-monitor-alignment-cxxqt-generated", kind = "static")]
    extern "C" {
        fn init_qt_resources();
    }

    unsafe {
        init_qt_resources();
    }
}
