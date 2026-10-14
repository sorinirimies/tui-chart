//! Sparkline integration tests.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Widget},
};
use tui_chart::Sparkline;

#[test]
fn sparkline_renders_blocks_in_block() {
    let area = Rect::new(0, 0, 8, 3);
    let mut buf = Buffer::empty(area);
    Sparkline::new([1.0, 5.0, 9.0])
        .block(Block::bordered())
        .render(area, &mut buf);
    assert_eq!(buf[(0, 0)].symbol(), "┌");
    assert_eq!(buf[(3, 1)].symbol(), "█");
}

#[test]
fn sparkline_empty_is_blank() {
    let area = Rect::new(0, 0, 4, 1);
    let mut buf = Buffer::empty(area);
    Sparkline::new(Vec::new()).render(area, &mut buf);
    assert_eq!(buf, Buffer::empty(area));
}
