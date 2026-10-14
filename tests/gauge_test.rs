//! Gauge integration tests.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_chart::{Gauge, GaugeStyle};

#[test]
fn gauge_fill_proportional() {
    let area = Rect::new(0, 0, 20, 1);
    let mut buf = Buffer::empty(area);
    Gauge::new(25.0).render(area, &mut buf);
    let full = (0..20).filter(|&x| buf[(x, 0)].symbol() == "█").count();
    assert_eq!(full, 5);
}

#[test]
fn gauge_arc_tiny_area_no_panic() {
    for (w, h) in [(0, 0), (1, 1), (2, 1), (30, 1)] {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        Gauge::new(50.0)
            .style(GaugeStyle::Arc)
            .render(area, &mut buf);
    }
}
