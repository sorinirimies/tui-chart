use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use tui_chart::{BarChart, BarGroup, BarMode, BarOrientation};

fn cells(buf: &Buffer, a: Rect, color: Color) -> Vec<(u16, u16)> {
    let mut v = vec![];
    for y in a.y..a.bottom() {
        for x in a.x..a.right() {
            let c = &buf[(x, y)];
            if c.fg == color && c.symbol() != " " && c.symbol() != "│" {
                v.push((x, y));
            }
        }
    }
    v
}

#[test]
fn negatives_extend_bounds() {
    let c = BarChart::new(vec![BarGroup::new("a", [-5.0]), BarGroup::new("b", [8.0])]);
    let (lo, hi) = c.bounds();
    assert!(lo <= -5.0 && hi >= 8.0);
}

#[test]
fn vertical_negative_below_positive() {
    let a = Rect::new(0, 0, 40, 20);
    let mut buf = Buffer::empty(a);
    BarChart::new(vec![
        BarGroup::new("p", [10.0]),
        BarGroup::new("n", [-10.0]),
    ])
    .colors(vec![Color::Red])
    .render(a, &mut buf);
    let r = cells(&buf, a, Color::Red);
    let xs: Vec<u16> = r.iter().map(|p| p.0).collect();
    let (min_x, max_x) = (*xs.iter().min().unwrap(), *xs.iter().max().unwrap());
    let left: Vec<u16> = r
        .iter()
        .filter(|p| p.0 < (min_x + max_x) / 2)
        .map(|p| p.1)
        .collect();
    let right: Vec<u16> = r
        .iter()
        .filter(|p| p.0 > (min_x + max_x) / 2)
        .map(|p| p.1)
        .collect();
    assert!(left.iter().max() < right.iter().min() || right.iter().max() < left.iter().min());
}

#[test]
fn horizontal_negative_left_of_positive() {
    let a = Rect::new(0, 0, 40, 12);
    let mut buf = Buffer::empty(a);
    BarChart::new(vec![
        BarGroup::new("p", [10.0]),
        BarGroup::new("n", [-10.0]),
    ])
    .orientation(BarOrientation::Horizontal)
    .colors(vec![Color::Red])
    .render(a, &mut buf);
    let r = cells(&buf, a, Color::Red);
    assert!(r.len() > 2);
    let top: Vec<u16> = r.iter().filter(|p| p.1 < 4).map(|p| p.0).collect();
    let bot: Vec<u16> = r.iter().filter(|p| p.1 >= 4).map(|p| p.0).collect();
    assert!(top.iter().max() <= bot.iter().min() || bot.iter().max() <= top.iter().min());
}

#[test]
fn stacked_mixed_signs() {
    let c = BarChart::new(vec![BarGroup::new("a", [4.0, -3.0, 2.0, -1.0])]).mode(BarMode::Stacked);
    let (lo, hi) = c.bounds();
    assert!(lo <= -4.0 && hi >= 6.0);
    let a = Rect::new(0, 0, 30, 16);
    let mut buf = Buffer::empty(a);
    c.show_values(true).render(a, &mut buf);
}
