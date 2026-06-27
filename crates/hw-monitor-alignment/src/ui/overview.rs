use windows_reactor::{
    Canvas, Color, Element, ElementExt as _, HorizontalAlignment, Shape, Stretch,
    VerticalAlignment, grid, text_block, viewbox,
};

use crate::monitor::Monitor;

// In real pixels
const MONITOR_BORDER_WIDTH: f64 = 20.0;

/// Extract the display index from a Win32 device name.
///
/// E.g. "\\.\DISPLAY1" -> "1".
fn index_from_device_name(device_name: &str) -> &str {
    let ummm = "?";

    match device_name.strip_prefix("\\\\.\\DISPLAY") {
        Some(stripped) => stripped,
        None => ummm,
    }
}

/// The virtual-screen overview.
///
/// Monitors are authored at their real pixel geometry and position, and then scaled by a [`ViewBox`].
pub fn overview_canvas(monitors: &[Monitor]) -> Element {
    let &[ref first, ref rest @ ..] = monitors else {
        return Canvas::new(std::iter::empty::<Element>())
            .background(Color::rgb(100, 100, 100))
            .into();
    };

    let (min_x, min_y, max_x, max_y) = rest.iter().fold(
        (
            first.x,
            first.y,
            first.x + first.width as i32,
            first.y + first.height as i32,
        ),
        |(min_x, min_y, max_x, max_y), m| {
            (
                min_x.min(m.x),
                min_y.min(m.y),
                max_x.max(m.x + m.width as i32),
                max_y.max(m.y + m.height as i32),
            )
        },
    );

    let total_width = f64::from(max_x - min_x);
    let total_height = f64::from(max_y - min_y);

    // fake monitors?
    if total_width == 0.0 || total_height == 0.0 {
        return Canvas::new(std::iter::empty::<Element>())
            .background(Color::rgb(100, 100, 100))
            .into();
    }

    let mut children: Vec<Element> = Vec::with_capacity(monitors.len() * 2);

    for monitor in monitors {
        // normalize the positions in a (0, 0) -> (total_width, total_height) plane
        let x_offset = f64::from(monitor.x - min_x);
        let y_offset = f64::from(monitor.y - min_y);

        let width = f64::from(monitor.width);
        let height = f64::from(monitor.height);

        let fill = if monitor.primary {
            // steel blue for primary
            Color::rgb(70, 130, 180)
        } else {
            // muted blue-gray for others
            Color::rgb(130, 130, 140)
        };

        let monitor_rectangle = Shape::rectangle()
            .fill(fill)
            .stroke(Color::rgb(235, 235, 235))
            .stroke_thickness(MONITOR_BORDER_WIDTH)
            .width(width)
            .height(height)
            .canvas_left(x_offset)
            .canvas_top(y_offset);

        children.push(monitor_rectangle.into());

        // in the same position as the monitor rectangle we draw a grid
        // in the grid a textbox (auto centered because of the grid)
        // with the display's number, scaled
        let label = index_from_device_name(&monitor.device_name);

        let font_size = height * 0.5;

        let number = grid([text_block(label)
            .font_size(font_size)
            .foreground(Color::rgb(245, 245, 245))
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center)])
        .width(width)
        .height(height)
        .canvas_left(x_offset)
        .canvas_top(y_offset);

        children.push(number.into());
    }

    let scene = Canvas::new(children)
        .width(total_width)
        .height(total_height);

    // the viewbox makes it so that the contents (the canvas) are scaled to the maximum boundaries
    viewbox(scene)
        .stretch(Stretch::Uniform)
        .horizontal_alignment(HorizontalAlignment::Center)
        .into()
}
