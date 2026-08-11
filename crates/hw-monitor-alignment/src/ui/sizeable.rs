use std::rc::Rc;

use windows_reactor::{
    Element, Grid, GridLength, HorizontalAlignment, LayoutExt as _, VerticalAlignment,
    composition_host, grid,
};

/// Wraps `child` so its rendered size is reported through `on_resize`.
pub struct Sizeable {
    child: Element,
    on_resize: Option<Rc<dyn Fn(f64, f64)>>,
}

impl Sizeable {
    pub fn new<I: Into<Element>>(child: I) -> Self {
        Self {
            child: child.into(),
            on_resize: None,
        }
    }

    pub fn on_resize<F: Fn(f64, f64) + 'static>(mut self, f: F) -> Self {
        self.on_resize = Some(Rc::new(f));
        self
    }

    /// The concrete wrapper grid, so callers can apply layout modifiers.
    pub fn into_grid(self) -> Grid {
        // An empty composition host is an invisible native grid; stretched to fill
        // the cell, its SizeChanged reports the cell size, which equals the
        // child's rendered size.
        let mut probe = composition_host()
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch);

        if let Some(on_resize) = self.on_resize {
            probe = probe.on_resize(move |w, h| on_resize(w, h));
        }

        // Both children share the grid's single cell (row/col 0):
        // the child renders, the probe overlays it and measures the same area.
        // column by default is Star, we want Auto
        // Star is 'take remaining space', Auto is size to child
        grid([self.child, probe.into()])
            .columns([GridLength::Auto])
            .rows([GridLength::Auto])
    }
}

pub fn sizeable<I: Into<Element>>(child: I) -> Sizeable {
    Sizeable::new(child)
}

impl From<Sizeable> for Element {
    fn from(value: Sizeable) -> Self {
        value.into_grid().into()
    }
}
