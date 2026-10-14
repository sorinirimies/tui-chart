//! Integration tests for `RenkoChart`.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{Block, Widget},
};
use tui_chart::{renko_bricks, RenkoChart, RenkoStyle};

fn closes() -> Vec<f64> {
    (0..80)
        .map(|i| 100.0 + (i as f64 / 5.0).sin() * 10.0)
        .collect()
}

#[test]
fn renko_draws_up_and_down_bricks() {
    assert!(!renko_bricks(&closes(), 1.0).is_empty());
    let area = Rect::new(0, 0, 60, 16);
    let mut buf = Buffer::empty(area);
    RenkoChart::new(&closes(), 1.0)
        .block(Block::bordered())
        .render(area, &mut buf);
    let mut colors = std::collections::HashSet::new();
    for y in 0..16 {
        for x in 0..60 {
            if buf[(x, y)].symbol() == "█" {
                colors.insert(buf[(x, y)].fg);
            }
        }
    }
    assert!(colors.contains(&Color::Green) && colors.contains(&Color::Red));
}

#[test]
fn renko_pf_and_tiny() {
    for (w, h) in [(0, 0), (2, 2), (30, 10)] {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        RenkoChart::new(&closes(), 1.0)
            .style(RenkoStyle::PointAndFigure)
            .render(area, &mut buf);
    }
}
