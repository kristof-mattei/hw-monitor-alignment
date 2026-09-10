use windows_reactor::{
    Border, ChildrenControl as _, Color, ContentControl as _, ElementRef, FontWeight, Grid,
    LayoutControl as _, Orientation, StackPanel, TextBlock, TextWrapping, Thickness, View,
};

use crate::monitor::Monitor;
use crate::ui::sizeable::sizeable;

/// Per-monitor info.
pub fn info_panel(monitors: &[Monitor], probe: &ElementRef<Grid>) -> View {
    // Show monitors left-to-right by virtual screen position.
    // TODO we should do this at the start
    let mut sorted: Vec<&Monitor> = monitors.iter().collect();
    sorted.sort_by_key(|m| (m.x, m.y));

    let boxes = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(8.0)
        .keyed_children(
            sorted
                .into_iter()
                .map(|m| (&*m.device_name, monitor_box(m))),
        );

    let outer = group_box("Monitor Setup Information", Thickness::xy(16.0, 0.0), boxes);

    // Resize observer on the OUTER panel.
    sizeable(outer, probe)
}

fn monitor_box(m: &Monitor) -> View {
    let fields = StackPanel::new().spacing(2.0).children([
        field("DEVICE NAME:", &m.device_name),
        field("MONITOR NAME:", &m.monitor_name),
        field("FRIENDLY MONITOR NAME:", &m.friendly_monitor_name),
        field("DISPLAY ADAPTER:", &m.display_adapter),
        field("SCREEN RESOLUTION:", &format!("{}x{}", m.width, m.height)),
        field("VIRTUAL SCREEN POSITION:", &format!("({}, {})", m.x, m.y)),
        field("ORIENTATION:", m.orientation.label()),
        field("IS PRIMARY:", if m.primary { "True" } else { "False" }),
    ]);

    group_box("Monitor Information", Thickness::uniform(0.0), fields)
}

fn field(caption: &str, value: &str) -> View {
    StackPanel::new().spacing(0.0).children((
        TextBlock::new().text(caption).font_size(11.0).opacity(0.6),
        TextBlock::new()
            .text(value)
            .font_size(12.0)
            .font_weight(FontWeight::BOLD)
            .text_wrapping(TextWrapping::Wrap),
    ))
}

fn group_box(title: &str, margin: Thickness, content: View) -> View {
    Border::new()
        .border_brush(Color::rgb(90, 90, 90))
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(4.0)
        .padding(Thickness::uniform(16.0))
        .margin(margin)
        .content(StackPanel::new().spacing(4.0).children((
            TextBlock::new().text(title).font_size(11.0).opacity(0.7),
            content,
        )))
}
