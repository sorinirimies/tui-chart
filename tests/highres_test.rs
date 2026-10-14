//! High-resolution marker tests across widgets.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_chart::{
    BarChart, BarGroup, BarOrientation, Canvas, DepthChart, Gauge, GaugeStyle, Histogram,
    LineChart, LineMode, Marker, RadarChart, RadarSeries, Series, Sparkline, SparklineStyle,
    Waterfall,
};

fn render(w: impl Widget, area: Rect) -> String {
    let mut buf = Buffer::empty(area);
    w.render(area, &mut buf);
    (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .map(|p| buf[p].symbol().to_string())
        .collect()
}

fn has(s: &str, lo: u32, hi: u32) -> bool {
    s.chars().any(|c| (lo..=hi).contains(&(c as u32)))
}
const BRAILLE: (u32, u32) = (0x2801, 0x28FF);
const SEXTANT: (u32, u32) = (0x1FB00, 0x1FB3B);

#[test]
fn sparkline_marker() {
    let a = Rect::new(0, 0, 20, 4);
    let mk = |m| {
        Sparkline::new((0..20).map(|i| i as f64))
            .style(SparklineStyle::Line)
            .marker(m)
    };
    assert!(has(&render(mk(Marker::Braille), a), BRAILLE.0, BRAILLE.1));
    assert!(has(&render(mk(Marker::Sextant), a), SEXTANT.0, SEXTANT.1));
    assert!(render(mk(Marker::Block), a).contains('█'));
}

#[test]
fn radar_marker() {
    let a = Rect::new(0, 0, 30, 15);
    let mk = |m| {
        RadarChart::new(["a", "b", "c", "d"])
            .series(RadarSeries::new("s", [1.0, 2.0, 3.0, 2.0]))
            .marker(m)
    };
    assert!(has(&render(mk(Marker::Braille), a), BRAILLE.0, BRAILLE.1));
    assert!(has(&render(mk(Marker::Sextant), a), SEXTANT.0, SEXTANT.1));
}

#[test]
fn gauge_marker() {
    let a = Rect::new(0, 0, 20, 6);
    let mk = |m| Gauge::new(50.0).style(GaugeStyle::Arc).marker(m);
    assert!(has(&render(mk(Marker::Braille), a), BRAILLE.0, BRAILLE.1));
    assert!(render(mk(Marker::HalfBlock), a).contains('▀'));
}

#[test]
fn histogram_kde_and_bar_markers() {
    let a = Rect::new(0, 0, 40, 12);
    let data = || (0..200).map(|i| ((i * 37) % 100) as f64 / 10.0);
    let kde = |m| Histogram::new(data()).bins(10).kde(true).marker(m);
    assert!(has(&render(kde(Marker::Braille), a), BRAILLE.0, BRAILLE.1));
    assert!(has(&render(kde(Marker::Sextant), a), SEXTANT.0, SEXTANT.1));
    let bars = Histogram::new(data()).bins(10).bar_marker(Marker::Braille);
    assert!(has(&render(bars, a), BRAILLE.0, BRAILLE.1));
}

#[test]
fn bar_marker_vertical_and_horizontal() {
    let a = Rect::new(0, 0, 30, 12);
    let groups = || {
        vec![
            BarGroup::new("a", [3.0]),
            BarGroup::new("b", [7.0]),
            BarGroup::new("c", [5.0]),
        ]
    };
    let default = render(BarChart::new(groups()), a);
    assert!(!has(&default, BRAILLE.0, BRAILLE.1));
    let v = render(BarChart::new(groups()).marker(Marker::Braille), a);
    assert!(has(&v, BRAILLE.0, BRAILLE.1));
    let h = render(
        BarChart::new(groups())
            .orientation(BarOrientation::Horizontal)
            .marker(Marker::Braille),
        a,
    );
    assert!(has(&h, BRAILLE.0, BRAILLE.1));
}

#[test]
fn depth_marker() {
    let a = Rect::new(0, 0, 40, 12);
    let book = || {
        DepthChart::new(
            (1..=10).map(|i| (100.0 - i as f64, i as f64)),
            (1..=10).map(|i| (100.0 + i as f64, i as f64 * 2.0)),
        )
    };
    assert!(!has(&render(book(), a), BRAILLE.0, BRAILLE.1));
    assert!(has(
        &render(book().marker(Marker::Braille), a),
        BRAILLE.0,
        BRAILLE.1
    ));
}

#[test]
fn waterfall_connector_marker() {
    let a = Rect::new(0, 0, 40, 12);
    let w = || {
        Waterfall::new()
            .delta("a", 10.0)
            .delta("b", -4.0)
            .delta("c", 6.0)
            .total("t")
    };
    assert!(render(w(), a).contains('┄'));
    let s = render(w().marker(Marker::Braille), a);
    assert!(!s.contains('┄') && has(&s, BRAILLE.0, BRAILLE.1));
}

#[test]
fn line_area_fill_uses_marker() {
    let a = Rect::new(0, 0, 40, 12);
    let mk = |m| {
        LineChart::new(vec![Series::new("s")
            .data((0..30).map(|i| (i as f64, (i as f64 / 4.0).sin() + 2.0)))
            .mode(LineMode::Area)])
        .marker(m)
    };
    let braille = render(mk(Marker::Braille), a);
    // area fill reaches low rows with full-height columns of dots
    assert!(braille.contains('\u{2805}'));
    // low-resolution markers fill solidly instead of stippling
    assert!(render(mk(Marker::Block), a).contains('█'));
    assert!(has(&render(mk(Marker::Sextant), a), SEXTANT.0, SEXTANT.1));
}

#[test]
fn canvas_reexports() {
    use tui_chart::OverlapPolicy;
    let mut c = Canvas::new(1, 1, Marker::Sextant);
    c.set_overlap(OverlapPolicy::FirstWins);
    assert_eq!(Marker::default(), Marker::Braille);
}
