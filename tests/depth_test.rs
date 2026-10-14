//! Integration tests for `DepthChart`.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{Block, Widget},
};
use tui_chart::DepthChart;

fn book() -> DepthChart {
    DepthChart::new(
        (1..=10).map(|i| (100.0 - i as f64, i as f64)),
        (1..=10).map(|i| (100.0 + i as f64, i as f64 * 2.0)),
    )
}

#[test]
fn depth_draws_bid_and_ask_areas() {
    let area = Rect::new(0, 0, 60, 14);
    let mut buf = Buffer::empty(area);
    book().block(Block::bordered()).render(area, &mut buf);
    let mut colors = std::collections::HashSet::new();
    for y in 0..14 {
        for x in 0..60 {
            if buf[(x, y)].symbol() == "█" {
                colors.insert(buf[(x, y)].fg);
            }
        }
    }
    assert!(colors.contains(&Color::Green) && colors.contains(&Color::Red));
}

#[test]
fn depth_tiny_and_empty_do_not_panic() {
    for (w, h) in [(0, 0), (2, 2), (10, 3)] {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        book().render(area, &mut buf);
        DepthChart::new([], []).render(area, &mut buf);
    }
}
