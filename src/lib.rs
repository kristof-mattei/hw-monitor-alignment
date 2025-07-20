#[expect(clippy::unnecessary_box_returns, reason = "3rd party code")]
pub mod cxxqt_object;

pub fn init_resources() {
    #[link(name = "hw-monitor-alignment-cxxqt-generated", kind = "static")]
    unsafe extern "C" {
        fn init_qt_resources();
    }

    // SAFETY: lib call
    unsafe {
        init_qt_resources();
    }
}
