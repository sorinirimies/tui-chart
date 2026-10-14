use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use tui_chart::*;

fn text(buf: &Buffer, a: Rect) -> String {
    (a.y..a.bottom())
        .flat_map(|y| (a.x..a.right()).map(move |x| (x, y)))
        .map(|p| buf[p].symbol().to_string())
        .collect()
}

#[test]
fn stream_into_line_chart_with_time_axis() {
    let mut s = Stream::new(50).x_start(1_700_000_000.0);
    for i in 0..200 {
        s.push((i as f64 / 7.0).sin());
    }
    let chart = LineChart::new(vec![s.series("v")]).x_axis(Axis::new().formatter(time_hms));
    let a = Rect::new(0, 0, 60, 14);
    let mut buf = Buffer::empty(a);
    chart.render(a, &mut buf);
    assert!(text(&buf, a).contains(':'));
}

#[test]
fn stack_panes_render_without_panic() {
    for h in [0u16, 1, 3, 20] {
        let area = Rect::new(0, 0, 30, h);
        for r in ChartStack::vertical().weights([2, 1]).gap(1).split(area) {
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 20));
            LineChart::new(vec![Series::new("a").values([1.0, 2.0])]).render(r, &mut buf);
        }
    }
}

#[test]
fn time_ticks_cover_range() {
    let t = time_ticks(1_700_000_000.0, 1_700_086_400.0, 6);
    assert!(t.len() >= 3);
    assert!(t
        .iter()
        .all(|&x| (1_700_000_000.0..=1_700_086_400.0).contains(&x)));
    assert_eq!(date_ymd(1_700_000_000.0), "2023-11-14");
}

#[test]
fn bar_colors_labels_and_negatives() {
    let groups = vec![
        BarGroup::new("a", [10.0, -5.0]).bar_color(0, Color::Red),
        BarGroup::new("b", [f64::NAN, 3.0]),
    ];
    for pos in [LabelPosition::Outside, LabelPosition::Inside] {
        for o in [BarOrientation::Vertical, BarOrientation::Horizontal] {
            let a = Rect::new(0, 0, 40, 12);
            let mut buf = Buffer::empty(a);
            BarChart::new(groups.clone())
                .orientation(o)
                .show_values(true)
                .label_position(pos)
                .render(a, &mut buf);
            assert!((0..12).any(|y| (0..40).any(|x| buf[(x, y)].fg == Color::Red)));
        }
    }
}

#[test]
fn histogram_modes() {
    let d: Vec<f64> = (0..300).map(|i| ((i * 13) % 97) as f64).collect();
    let h = Histogram::new(d).bins(8);
    let c = h.clone().cumulative(true).heights();
    assert_eq!(*c.last().unwrap(), 300.0);
    assert!(!h.kde_curve(40, None).is_empty());
    let a = Rect::new(0, 0, 40, 12);
    let mut buf = Buffer::empty(a);
    h.normalized(true).cumulative(true).render(a, &mut buf);
}
