use hw_monitor_alignment::init_resources;

#[test]
fn init() {
    // IF YOU REMOVE THIS LINE IT'LL BREAK
    init_resources();
}

#[test]
fn assert_world_ok() {
    let cls1 = || true;
    let cls2 = || true;
    assert_eq!(cls1(), cls2());
}
