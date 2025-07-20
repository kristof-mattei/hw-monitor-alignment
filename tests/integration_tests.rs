#![expect(clippy::tests_outside_test_module, reason = "Integration tests")]

#[expect(
    unused,
    reason = "To ensure QT stuff gets included when compiling for tests"
)]
use hw_monitor_alignment::init_resources;

#[test]
fn assert_world_ok() {
    let cls1 = || true;
    let cls2 = || true;
    assert_eq!(cls1(), cls2());
}
