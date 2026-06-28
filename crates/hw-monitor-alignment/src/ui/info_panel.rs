use windows_reactor::{
    Border, Color, Element, ElementExt as _, Thickness, border, hstack, text_block, vstack,
};

use crate::monitor::Monitor;

/// Per-monitor info.
pub fn info_panel(monitors: &[Monitor]) -> Element {
    // Show monitors left-to-right by virtual screen position.
    // TODO we should do this at the start
    let mut sorted: Vec<&Monitor> = monitors.iter().collect();
    sorted.sort_by_key(|m| (m.x, m.y));

    let boxes: Vec<Element> = sorted.iter().map(|m| monitor_box(m).into()).collect();

    group_box(
        "Monitor Setup Information",
        hstack(boxes).spacing(8.0).into(),
    )
    .margin(Thickness::xy(16.0, 0.0))
    .into()
}

fn monitor_box(m: &Monitor) -> impl Into<Element> {
    let fields = vstack([
        field("DEVICE NAME:", &m.device_name),
        field("MONITOR NAME:", &m.monitor_name),
        field("FRIENDLY MONITOR NAME:", &m.friendly_monitor_name),
        field("DISPLAY ADAPTER:", &m.display_adapter),
        field("SCREEN RESOLUTION:", &format!("{}x{}", m.width, m.height)),
        field("VIRTUAL SCREEN POSITION:", &format!("({}, {})", m.x, m.y)),
        field("ORIENTATION:", m.orientation.label()),
        field("IS PRIMARY:", if m.primary { "True" } else { "False" }),
    ])
    .spacing(2.0);

    group_box("Monitor Information", fields.into())
}

fn field(caption: &str, value: &str) -> Element {
    vstack([
        text_block(caption.to_owned()).font_size(11.0).opacity(0.6),
        text_block(value.to_owned()).font_size(12.0).bold().wrap(),
    ])
    .spacing(0.0)
    .into()
}

fn group_box(title: &str, content: Element) -> Border {
    let inner = vstack((
        text_block(title.to_owned()).font_size(11.0).opacity(0.7),
        content,
    ))
    .spacing(4.0)
    .padding(Thickness::uniform(8.0));

    border(inner)
        .border_brush(Color::rgb(90, 90, 90))
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(4.0)
        .padding(Thickness::uniform(8.0))
}
