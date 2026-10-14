#![no_main]
mod common;
use arbitrary::Arbitrary;
use common::*;
use libfuzzer_sys::fuzz_target;
use tui_chart::*;

#[derive(Arbitrary, Debug)]
struct SeriesIn {
    name: String,
    points: Vec<(f64, f64)>,
    mode: u8,
    smooth: bool,
    secondary: bool,
    err: Vec<f64>,
    band: Vec<(f64, f64)>,
    color: Col,
}

#[derive(Arbitrary, Debug)]
struct Input {
    series: Vec<SeriesIn>,
    x: AxisIn,
    y: AxisIn,
    y2: Option<AxisIn>,
    log: bool,
    smooth: bool,
    marker: MarkerIn,
    mode: u8,
    legend: u8,
    theme: u8,
    crosshair: Option<(f64, f64)>,
    annotations: Vec<(u8, f64, f64)>,
    size: Size,
}

fn mode(m: u8) -> LineMode {
    match m % 4 {
        0 => LineMode::Line,
        1 => LineMode::Step,
        2 => LineMode::Area,
        _ => LineMode::Scatter,
    }
}

fuzz_target!(|i: Input| {
    let series: Vec<Series> = i
        .series
        .iter()
        .take(8)
        .map(|s| {
            Series::new(s.name.clone())
                .data(s.points.iter().copied().take(MAX_POINTS))
                .mode(mode(s.mode))
                .smooth(s.smooth)
                .secondary_axis(s.secondary)
                .y_err(s.err.iter().copied().take(MAX_POINTS))
                .band(s.band.iter().copied().take(MAX_POINTS))
                .color(s.color.get())
        })
        .collect();
    let mut c = LineChart::new(series)
        .x_axis(i.x.get())
        .y_axis(i.y.get())
        .y_log(i.log)
        .smooth(i.smooth)
        .marker(i.marker.get())
        .mode(mode(i.mode))
        .theme(theme(i.theme))
        .legend(match i.legend % 5 {
            0 => LegendPosition::None,
            1 => LegendPosition::TopLeft,
            2 => LegendPosition::TopRight,
            3 => LegendPosition::BottomLeft,
            _ => LegendPosition::BottomRight,
        });
    if let Some(a) = &i.y2 {
        c = c.y2_axis(a.get());
    }
    if let Some((x, y)) = i.crosshair {
        c = c.crosshair(Some(Crosshair::new(x, y)));
        let _ = c.nearest(x, y);
    }
    for (k, a, b) in i.annotations.iter().take(16) {
        c = c.annotation(match k % 5 {
            0 => Annotation::hline(*a),
            1 => Annotation::vline(*a),
            2 => Annotation::x_region(*a, *b),
            3 => Annotation::y_region(*a, *b),
            _ => Annotation::label(*a, *b, "n"),
        });
    }
    let _ = (c.bounds(), c.y2_bounds());
    render(c, i.size);
});
