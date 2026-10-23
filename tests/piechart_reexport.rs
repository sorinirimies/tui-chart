#![cfg(feature = "piechart")]
//! `tui-piechart` is re-exported so one dependency covers charts and pies.

use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use tui_chart::{PieChart, PieSlice};

#[test]
fn pie_chart_renders_through_reexport() {
    let area = Rect::new(0, 0, 40, 12);
    let mut buf = Buffer::empty(area);
    PieChart::new(vec![
        PieSlice::new("Rust", 60.0, Color::Red),
        PieSlice::new("Go", 40.0, Color::Blue),
    ])
    .render(area, &mut buf);
    assert!(buf.content().iter().any(|c| c.symbol() != " "));
}

#[test]
fn whole_crate_is_reachable() {
    let _ = tui_chart::piechart::Resolution::Standard;
}
