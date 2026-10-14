use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use tui_chart::{draw_annotations, nearest_point, Annotation, Axis, Crosshair, LineChart, Series};

fn text(buf: &Buffer, a: Rect) -> String {
    (0..a.height)
        .flat_map(|y| (0..a.width).map(move |x| (x, y)))
        .map(|p| buf[p].symbol().to_string())
        .collect()
}

fn base() -> LineChart {
    LineChart::new(vec![
        Series::new("a").data((0..30).map(|i| (i as f64, (i * i) as f64 + 1.0)))
    ])
}

#[test]
fn log_axis_shows_decades() {
    let a = Rect::new(0, 0, 50, 16);
    let mut buf = Buffer::empty(a);
    base().y_log(true).render(a, &mut buf);
    let t = text(&buf, a);
    assert!(t.contains("100"));
    assert!(t.contains("1000"));
}

#[test]
fn annotations_visible() {
    let a = Rect::new(0, 0, 50, 16);
    let mut buf = Buffer::empty(a);
    base()
        .annotation(Annotation::hline(100.0).text("limit"))
        .annotation(Annotation::x_region(5.0, 10.0).color(Color::Blue))
        .render(a, &mut buf);
    let t = text(&buf, a);
    assert!(t.contains("limit") && t.contains('╌'));
    assert!((0..a.width).any(|x| buf[(x, 5)].bg == Color::Blue));
}

#[test]
fn secondary_axis_draws_right_labels() {
    let a = Rect::new(0, 0, 50, 12);
    let mut buf = Buffer::empty(a);
    LineChart::new(vec![
        Series::new("l").values([1.0, 2.0, 3.0]),
        Series::new("r")
            .values([5000.0, 7000.0, 9000.0])
            .secondary_axis(true),
    ])
    .y2_axis(Axis::new())
    .render(a, &mut buf);
    assert!(text(&buf, a).contains("9000"));
}

#[test]
fn crosshair_and_nearest() {
    let c = base().crosshair(Some(Crosshair::new(10.0, 500.0)));
    let a = Rect::new(0, 0, 60, 18);
    let mut buf = Buffer::empty(a);
    c.render(a, &mut buf);
    assert!(text(&buf, a).contains('┼'));
    let n = base().nearest(10.0, 101.0).unwrap();
    assert_eq!(n.index, 10);
    assert_eq!(
        nearest_point([(0.0, 0.0), (1.0, 1.0)], (0.9, 0.9), (1.0, 1.0))
            .unwrap()
            .0,
        1
    );
}

#[test]
fn smooth_band_err_tiny_and_nan() {
    let s = Series::new("s")
        .values([1.0, f64::NAN, 3.0, 2.0])
        .smooth(true)
        .y_err([0.1, 0.2])
        .band([(0.0, 2.0)]);
    for (w, h) in [(0, 0), (3, 2), (20, 8), (80, 24)] {
        let a = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(a);
        LineChart::new(vec![s.clone()])
            .y_log(true)
            .crosshair(Some(Crosshair::new(f64::NAN, 1.0)))
            .render(a, &mut buf);
    }
}

#[test]
fn generic_annotation_drawing() {
    let plot = Rect::new(2, 1, 10, 5);
    let mut buf = Buffer::empty(Rect::new(0, 0, 14, 7));
    draw_annotations(
        &mut buf,
        plot,
        &[Annotation::label(1.0, 1.0, "hi")],
        &|x| Some(x as i32),
        &|y| Some(y as i32),
    );
    assert_eq!(buf[(3, 2)].symbol(), "h");
}
