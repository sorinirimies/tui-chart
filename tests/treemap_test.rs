//! Integration tests for `Treemap`.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Widget},
};
use tui_chart::{Treemap, TreemapItem};

#[test]
fn treemap_fills_area_and_shows_labels() {
    let area = Rect::new(0, 0, 50, 16);
    let mut buf = Buffer::empty(area);
    Treemap::new([
        TreemapItem::new("Rust", 40.0),
        TreemapItem::new("Go", 25.0),
        TreemapItem::new("Zig", 10.0),
        TreemapItem::new("C", 5.0),
    ])
    .render(area, &mut buf);
    let t: String = (0..16)
        .flat_map(|y| (0..50).map(move |x| (x, y)))
        .map(|p| buf[p].symbol().to_string())
        .collect();
    assert!(t.contains("Rust") && t.contains("Go"));
    assert!((0..50).all(|x| (0..16).all(|y| buf[(x, y)].bg != ratatui::style::Color::Reset)));
}

#[test]
fn treemap_degenerate_inputs() {
    for (w, h) in [(0, 0), (1, 1), (4, 2)] {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        Treemap::new([TreemapItem::new("a", f64::NAN), TreemapItem::new("b", 0.0)])
            .block(Block::bordered())
            .render(area, &mut buf);
    }
}
