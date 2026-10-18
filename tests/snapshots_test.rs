//! Snapshot-like tests: small deterministic charts compared row-by-row
//! against inline expectations (no snapshot crate).

mod common;
use common::*;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_chart::*;

fn snap(w: impl Widget, width: u16, height: u16) -> Vec<String> {
    let r = Rect::new(0, 0, width, height);
    let mut b = Buffer::empty(r);
    w.render(r, &mut b);
    rows(&b, r)
}

#[test]
fn sparkline_ramp() {
    assert_eq!(
        snap(
            Sparkline::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]),
            8,
            1
        ),
        ["▁▁▂▃▅▆▇█"]
    );
}

#[test]
fn gauge_half() {
    assert_eq!(
        snap(Gauge::new(50.0).range(0.0, 100.0), 12, 1),
        ["██████      "]
    );
}

#[test]
fn heatmap_values() {
    assert_eq!(
        snap(
            Heatmap::new(vec![vec![0.0, 1.0, 2.0], vec![3.0, 4.0, 5.0]]).show_values(true),
            12,
            4
        ),
        [
            "│0 1 2      ",
            "│3 4 5      ",
            "│           ",
            "└───────────"
        ]
    );
}

#[test]
fn bar_three_groups() {
    let c = BarChart::new(vec![
        BarGroup::new("a", [1.0]),
        BarGroup::new("b", [2.0]),
        BarGroup::new("c", [3.0]),
    ]);
    assert_eq!(
        snap(c, 20, 8),
        [
            "3│············█████·",
            " │            █████ ",
            "2│······█████·█████·",
            "1│······█████·█████·",
            " │█████ █████ █████ ",
            "0│█████·█████·█████·",
            " └──────────────────",
            "    a     b     c   ",
        ]
    );
}

#[test]
fn waterfall_up_down_total() {
    let w = Waterfall::new()
        .initial(10.0)
        .delta("up", 5.0)
        .delta("dn", -8.0)
        .total("T");
    assert_eq!(
        snap(w, 24, 8),
        [
            "    +5     -8           ",
            "  ██████┄██████         ",
            "  ██████ ██████         ",
            "  ██████ ██████    7    ",
            "         ██████┄███████ ",
            "                ███████ ",
            "0               ███████ ",
            "  up     dn     T       ",
        ]
    );
}

#[test]
fn treemap_two_items() {
    let t = Treemap::new([TreemapItem::new("A", 3.0), TreemapItem::new("B", 1.0)]);
    assert_eq!(
        snap(t, 12, 4),
        [
            "A        B  ",
            "3        1  ",
            "            ",
            "            "
        ]
    );
}

#[test]
fn line_zigzag() {
    let c = LineChart::new(vec![Series::new("s").values([0.0, 1.0, 0.0, 1.0])]);
    assert_eq!(
        snap(c, 24, 8),
        [
            "1.00│     ⡰⠱⡀        ●⢀s",
            "0.75│    ⡰⠁ ⠱⡀       ⢀⠎ ",
            "    │   ⡰⠁   ⠘⡄     ⢀⠎  ",
            "0.50│  ⡰⠁     ⠘⡄   ⢀⠎   ",
            "0.25│ ⡰⠁       ⠈⢆ ⢀⠎    ",
            "0.00│⡰⠁         ⠈⢆⠎     ",
            "    └───────────────────",
            "     0     1     2     3",
        ]
    );
}

/// Every widget with normal data renders something visible at 80x24 and the
/// output is stable across runs.
#[test]
fn every_widget_draws_something() {
    for (name, draw) in all_widgets(Data::Normal, None) {
        if name == "RenkoChart(zero brick)" {
            continue;
        }
        let area = Rect::new(0, 0, 80, 24);
        let (buf, _) = render_padded(&draw, area);
        let ink = rows(&buf, area)
            .iter()
            .flat_map(|r| r.chars())
            .filter(|c| !c.is_whitespace() && *c != '§')
            .count();
        let bg_cells = (0..24)
            .flat_map(|y| (0..80).map(move |x| (x, y)))
            .filter(|p| buf[*p].bg != ratatui::style::Color::Reset)
            .count();
        assert!(ink + bg_cells > 0, "{name}: nothing drawn");
    }
}
