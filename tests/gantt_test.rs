//! Integration tests for Gantt.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_chart::*;

#[test]
fn renders_tasks_and_today() {
    let area = Rect::new(0, 0, 50, 8);
    let mut buf = Buffer::empty(area);
    Gantt::new([
        Task::new("one", 0.0, 10.0).progress(0.5),
        Task::new("two", 5.0, 20.0),
    ])
    .today(8.0)
    .render(area, &mut buf);
    let all: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(all.contains("one") && all.contains("two") && all.contains('█') && all.contains('┃'));
}

#[test]
fn zero_area_ok() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 5, 5));
    Gantt::new([Task::new("a", 0.0, 1.0)]).render(Rect::new(0, 0, 0, 0), &mut buf);
}
