//! Integration tests for Waterfall.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_chart::*;

#[test]
fn renders_values_and_labels() {
    let area = Rect::new(0, 0, 40, 14);
    let mut buf = Buffer::empty(area);
    Waterfall::new()
        .delta("Up", 50.0)
        .delta("Dn", -20.0)
        .total("Sum")
        .render(area, &mut buf);
    let all: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(all.contains("+50") && all.contains("-20") && all.contains("Sum"));
}

#[test]
fn zero_area_ok() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 5, 5));
    Waterfall::new()
        .delta("a", 1.0)
        .render(Rect::new(0, 0, 0, 0), &mut buf);
}
