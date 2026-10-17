//! Quick render timing: `cargo run --release --example bench`.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use std::time::Instant;
use tui_chart::*;

fn time(name: &str, n: u32, mut f: impl FnMut()) {
    f();
    let t = Instant::now();
    for _ in 0..n {
        f();
    }
    println!("{name:<28} {:>9.1?} / frame", t.elapsed() / n);
}

fn main() {
    let area = Rect::new(0, 0, 200, 50);
    let pts: Vec<(f64, f64)> = (0..100_000)
        .map(|i| (i as f64, (i as f64 / 300.0).sin() * 50.0 + (i % 17) as f64))
        .collect();
    let line = LineChart::new(vec![Series::new("a").data(pts.clone())]);
    time("line 100k pts", 20, || {
        let mut b = Buffer::empty(area);
        (&line).render(area, &mut b);
    });
    let area_chart = LineChart::new(vec![Series::new("a").data(pts.clone())]).mode(LineMode::Area);
    time("area 100k pts", 20, || {
        let mut b = Buffer::empty(area);
        (&area_chart).render(area, &mut b);
    });
    let candles: Vec<Candle> = (0..5000)
        .map(|i| {
            let b = 100.0 + (i as f64 / 20.0).sin() * 10.0;
            Candle::new(b, b + 3.0, b - 3.0, b + 1.0).volume(100.0 + (i % 7) as f64)
        })
        .collect();
    let c = CandlestickChart::new(candles).show_volume(true);
    time("candles 5k", 20, || {
        let mut b = Buffer::empty(area);
        (&c).render(area, &mut b);
    });
    let rows: Vec<Vec<f64>> = (0..200)
        .map(|y| (0..400).map(|x| ((x * y) % 97) as f64).collect())
        .collect();
    let h = Heatmap::new(rows);
    time("heatmap 400x200", 20, || {
        let mut b = Buffer::empty(area);
        (&h).render(area, &mut b);
    });
    let s = Sparkline::new((0..100_000).map(|i| (i % 50) as f64));
    time("sparkline 100k", 20, || {
        let mut b = Buffer::empty(area);
        (&s).render(area, &mut b);
    });
}
