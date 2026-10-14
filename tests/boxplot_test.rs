//! Integration tests for BoxPlot.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_chart::*;

#[test]
fn renders_groups_both_orientations() {
    for o in [BoxOrientation::Vertical, BoxOrientation::Horizontal] {
        let area = Rect::new(0, 0, 50, 16);
        let mut buf = Buffer::empty(area);
        BoxPlot::new()
            .orientation(o)
            .group("a", (0..50).map(|x| x as f64))
            .stats("b", BoxStats::new(5.0, 10.0, 20.0, 30.0, 45.0))
            .render(area, &mut buf);
        assert!(buf.content().iter().any(|c| c.symbol() == "█"));
    }
}

#[test]
fn zero_area_ok() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 5, 5));
    BoxPlot::new()
        .group("a", [1.0])
        .render(Rect::new(0, 0, 0, 0), &mut buf);
}
