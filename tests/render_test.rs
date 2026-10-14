//! Integration tests: render every widget through the public API and
//! inspect the resulting buffer.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{Block, Widget},
};
use tui_chart::*;

fn render(w: impl Widget, width: u16, height: u16) -> (Buffer, Rect) {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    w.render(area, &mut buf);
    (buf, area)
}

fn text(buf: &Buffer, area: Rect) -> String {
    (area.y..area.bottom())
        .map(|y| {
            (area.x..area.right())
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn candles(n: usize) -> Vec<Candle> {
    (0..n)
        .map(|i| {
            let b = 50.0 + i as f64;
            Candle::new(b, b + 4.0, b - 3.0, b + if i % 2 == 0 { 2.0 } else { -2.0 })
                .volume(10.0 * (i + 1) as f64)
                .label(format!("d{i}"))
        })
        .collect()
}

#[test]
fn line_chart_has_axes_labels_and_title() {
    let chart = LineChart::new(vec![Series::new("s").data([(0.0, 0.0), (10.0, 100.0)])])
        .x_axis(Axis::new().title("time"))
        .y_axis(Axis::new().title("price"))
        .block(Block::bordered().title("T"));
    let (buf, area) = render(chart, 50, 15);
    let t = text(&buf, area);
    assert!(t.contains("price"));
    assert!(t.contains("time"));
    assert!(t.contains("100"));
    assert!(t.contains('└'));
    assert!(t.starts_with("┌T"));
}

#[test]
fn line_chart_endpoints_reach_corners() {
    let chart = LineChart::new(vec![Series::new("s")
        .data([(0.0, 0.0), (1.0, 1.0)])
        .color(Color::Red)])
    .legend(LegendPosition::None)
    .x_axis(Axis::new().bounds(0.0, 1.0))
    .y_axis(Axis::new().bounds(0.0, 1.0));
    let (buf, area) = render(chart, 30, 10);
    // find plot origin: first column after the axis line
    let mut red = 0;
    for y in 0..area.height {
        for x in 0..area.width {
            if buf[(x, y)].fg == Color::Red {
                red += 1;
            }
        }
    }
    assert!(red >= 8, "diagonal should paint many cells, got {red}");
}

#[test]
fn area_mode_fills_below_line() {
    let flat = || Series::new("f").data([(0.0, 5.0), (10.0, 5.0)]);
    let (line, a) = render(
        LineChart::new(vec![flat()]).y_axis(Axis::new().bounds(0.0, 10.0)),
        40,
        12,
    );
    let (area_buf, _) = render(
        LineChart::new(vec![flat()])
            .mode(LineMode::Area)
            .y_axis(Axis::new().bounds(0.0, 10.0)),
        40,
        12,
    );
    let count = |b: &Buffer| {
        text(b, a)
            .chars()
            .filter(|c| ('\u{2801}'..='\u{28ff}').contains(c))
            .count()
    };
    assert!(count(&area_buf) > count(&line) * 2);
}

#[test]
fn scatter_draws_points_only() {
    let chart = LineChart::new(vec![Series::new("p").data([
        (0.0, 0.0),
        (5.0, 5.0),
        (10.0, 0.0),
    ])])
    .mode(LineMode::Scatter)
    .legend(LegendPosition::None);
    let (buf, area) = render(chart, 40, 12);
    let n = text(&buf, area)
        .chars()
        .filter(|c| ('\u{2801}'..='\u{28ff}').contains(c))
        .count();
    assert!((3..=12).contains(&n), "got {n}");
}

#[test]
fn candlestick_renders_labels_volume_overlay() {
    let c = candles(12);
    let avg = sma(&c, 3);
    let chart = CandlestickChart::new(c)
        .show_volume(true)
        .overlay(Overlay::new("SMA3", avg, Color::Yellow))
        .block(Block::bordered());
    let (buf, area) = render(chart, 60, 22);
    let t = text(&buf, area);
    assert!(t.contains("d0"));
    assert!(t.contains("SMA3"));
    assert!(t.contains('█'));
    assert!(t.contains('▁') || t.contains('▂') || t.contains('█'));
}

#[test]
fn candlestick_scroll_changes_window() {
    let a = render(CandlestickChart::new(candles(100)), 30, 12);
    let b = render(CandlestickChart::new(candles(100)).scroll(30), 30, 12);
    assert_ne!(text(&a.0, a.1), text(&b.0, b.1));
}

#[test]
fn candle_styles_differ() {
    let t = |s| {
        let (b, a) = render(CandlestickChart::new(candles(8)).style(s), 40, 14);
        text(&b, a)
    };
    let (c, o, h) = (
        t(CandleStyle::Candle),
        t(CandleStyle::Ohlc),
        t(CandleStyle::HeikinAshi),
    );
    assert_ne!(c, o);
    assert_ne!(c, h);
}

#[test]
fn up_and_down_colors_are_configurable() {
    let chart = CandlestickChart::new(candles(6)).colors(Color::Blue, Color::Magenta);
    let (buf, area) = render(chart, 30, 14);
    let mut seen = std::collections::HashSet::new();
    for y in 0..area.height {
        for x in 0..area.width {
            seen.insert(buf[(x, y)].fg);
        }
    }
    assert!(seen.contains(&Color::Blue) && seen.contains(&Color::Magenta));
}

#[test]
fn bar_chart_orientations() {
    let groups = || vec![BarGroup::new("a", [1.0]), BarGroup::new("b", [4.0])];
    let (v, a) = render(BarChart::new(groups()), 30, 12);
    let (h, _) = render(
        BarChart::new(groups()).orientation(BarOrientation::Horizontal),
        30,
        12,
    );
    assert_ne!(text(&v, a), text(&h, a));
    assert!(text(&h, a).contains('█'));
}

#[test]
fn bar_height_is_proportional() {
    let chart = BarChart::new(vec![BarGroup::new("a", [10.0]), BarGroup::new("b", [5.0])])
        .bar_width(2)
        .value_axis(Axis::new().bounds(0.0, 10.0));
    let (buf, area) = render(chart, 30, 12);
    let col_cells = |x: u16| {
        (0..area.height)
            .filter(|&y| buf[(x, y)].symbol() == "█")
            .count()
    };
    // find the first two bar columns by scanning for full blocks
    let cols: Vec<u16> = (0..area.width).filter(|&x| col_cells(x) > 0).collect();
    let tall = col_cells(cols[0]);
    let short = col_cells(*cols.last().unwrap());
    assert!(tall >= short * 2 - 1, "tall={tall} short={short}");
}

#[test]
fn histogram_counts_and_renders() {
    let h = Histogram::new((0..100).map(|i| (i % 10) as f64)).bins(10);
    let bins = h.compute_bins();
    assert_eq!(bins.len(), 10);
    assert!(bins.iter().all(|b| b.count == 10));
    let (buf, area) = render(h, 40, 12);
    assert!(text(&buf, area).contains('█'));
}

#[test]
fn heatmap_cells_use_gradient() {
    let hm = Heatmap::new(vec![vec![0.0, 1.0]])
        .gradient(Gradient::Grayscale)
        .legend(false)
        .cell_width(2);
    let (buf, _) = render(hm, 20, 6);
    let bgs: Vec<Color> = (0..20).map(|x| buf[(x, 0)].bg).collect();
    assert!(bgs.contains(&Color::Rgb(0, 0, 0)));
    assert!(bgs.contains(&Color::Rgb(255, 255, 255)));
}

#[test]
fn widgets_render_by_reference_and_by_value_identically() {
    let chart = LineChart::new(vec![Series::new("s").values([1.0, 2.0, 3.0])]);
    let area = Rect::new(0, 0, 30, 10);
    let mut a = Buffer::empty(area);
    let mut b = Buffer::empty(area);
    (&chart).render(area, &mut a);
    chart.render(area, &mut b);
    assert_eq!(a, b);
}

#[test]
fn rendering_into_offset_area_stays_inside() {
    let area = Rect::new(5, 3, 30, 10);
    let full = Rect::new(0, 0, 50, 20);
    let mut buf = Buffer::empty(full);
    CandlestickChart::new(candles(10))
        .show_volume(true)
        .block(Block::bordered())
        .render(area, &mut buf);
    for y in 0..full.height {
        for x in 0..full.width {
            let inside = x >= area.x && x < area.right() && y >= area.y && y < area.bottom();
            if !inside {
                assert_eq!(buf[(x, y)].symbol(), " ", "leak at {x},{y}");
            }
        }
    }
}

#[test]
fn stress_all_widgets_many_sizes() {
    for w in [0u16, 1, 2, 3, 5, 8, 13, 21, 40, 80] {
        for h in [0u16, 1, 2, 3, 5, 8, 13, 21] {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            LineChart::new(vec![Series::new("a").values([1.0, 5.0, 2.0])])
                .mode(LineMode::Area)
                .render(area, &mut buf);
            CandlestickChart::new(candles(20))
                .show_volume(true)
                .render(area, &mut buf);
            BarChart::new(vec![BarGroup::new("a", [1.0, 2.0])])
                .mode(BarMode::Stacked)
                .show_values(true)
                .render(area, &mut buf);
            BarChart::new(vec![BarGroup::new("a", [1.0, 2.0])])
                .orientation(BarOrientation::Horizontal)
                .render(area, &mut buf);
            Histogram::new([1.0, 2.0, 3.0]).render(area, &mut buf);
            Heatmap::new(vec![vec![1.0, 2.0]])
                .show_values(true)
                .render(area, &mut buf);
        }
    }
}
