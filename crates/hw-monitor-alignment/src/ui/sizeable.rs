use windows_reactor::{
    ChildrenControl as _, ElementRef, Grid, GridLength, HorizontalAlignment, LayoutControl as _,
    VerticalAlignment, View,
};

/// Wraps `child` so [`ElementRef::observe_composition_host`] on `probe` reports its size.
pub fn sizeable<I: Into<View>>(child: I, probe: &ElementRef<Grid>) -> View {
    // An empty composition host is an invisible native grid; stretched to fill
    // the cell, its metrics report the cell size, which equals the child's
    // rendered size.
    let host = Grid::new()
        .element_ref(probe)
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .vertical_alignment(VerticalAlignment::Stretch);

    // Both children share the grid's single cell (row/col 0):
    // the child renders, the probe overlays it and measures the same area.
    // column by default is Star, we want Auto
    // Star is 'take remaining space', Auto is size to child
    Grid::new()
        .columns([GridLength::Auto])
        .rows([GridLength::Auto])
        .children((child, host))
}
