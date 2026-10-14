//! RadarChart integration tests.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_chart::{RadarChart, RadarSeries};

#[test]
fn radar_shows_axis_labels() {
    let area = Rect::new(0, 0, 50, 20);
    let mut buf = Buffer::empty(area);
    RadarChart::new(["North", "East", "South", "West"])
        .series(RadarSeries::new("s", [1.0, 2.0, 3.0, 4.0]))
        .render(area, &mut buf);
    let t: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(t.contains("North") && t.contains("West"));
}

#[test]
fn radar_no_series_ok() {
    let area = Rect::new(0, 0, 10, 6);
    let mut buf = Buffer::empty(area);
    (&RadarChart::new(["a", "b", "c"])).render(area, &mut buf);
}
