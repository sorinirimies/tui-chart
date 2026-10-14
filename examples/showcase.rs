//! Dashboard showing every chart type at once.
//! Keys: space pause · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::Color,
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{
    sma, Axis, BarChart, BarGroup, BarMode, Candle, CandlestickChart, Gradient, Heatmap, Histogram,
    LineChart, LineMode, Overlay, Series, Theme,
};

struct State {
    theme: Theme,
    t: f64,
    rng: Rng,
    candles: Vec<Candle>,
    samples: Vec<f64>,
    paused: bool,
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(11);
    let mut candles = vec![];
    let mut p = 100.0;
    for i in 0..80 {
        let o = p;
        let c = o + rng.gauss();
        candles.push(
            Candle::new(o, o.max(c) + rng.f64(), o.min(c) - rng.f64(), c)
                .volume(1000.0 + rng.f64() * 1000.0)
                .label(format!("{i}")),
        );
        p = c;
    }
    let samples = (0..500).map(|_| rng.gauss()).collect();
    run(
        State {
            theme: Theme::default(),
            t: 0.0,
            rng,
            candles,
            samples,
            paused: false,
        },
        Duration::from_millis(200),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(10), Constraint::Length(1)]).areas(f.area());
            let [top, bottom] =
                Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)])
                    .areas(main);
            let [a, b] =
                Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
                    .areas(top);
            let [c, d, e] = Layout::horizontal([Constraint::Ratio(1, 3); 3]).areas(bottom);

            let candle = CandlestickChart::new(s.candles.clone())
                .show_volume(true)
                .overlay(Overlay::new("SMA10", sma(&s.candles, 10), Color::Yellow))
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Candlestick "));
            f.render_widget(&candle, a);

            let area_chart = LineChart::new(vec![
                Series::new("load")
                    .data((0..60).map(|i| (i as f64, ((i as f64 + s.t) / 6.0).sin() * 3.0 + 5.0))),
                Series::new("mem")
                    .data((0..60).map(|i| (i as f64, ((i as f64 + s.t) / 9.0).cos() * 2.0 + 4.0)))
                    .mode(LineMode::Step),
            ])
            .mode(LineMode::Area)
            .y_axis(Axis::new().grid(true))
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Area / Step "));
            f.render_widget(&area_chart, b);

            let bars = BarChart::new(vec![
                BarGroup::new("A", [4.0, 2.0]),
                BarGroup::new("B", [7.0, 3.0]),
                BarGroup::new("C", [5.0, 6.0]),
            ])
            .mode(BarMode::Stacked)
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Stacked bars "));
            f.render_widget(&bars, c);

            let hist = Histogram::new(s.samples.clone())
                .bins(10)
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Histogram "));
            f.render_widget(&hist, d);

            let grid: Vec<Vec<f64>> = (0..6)
                .map(|r| {
                    (0..10)
                        .map(|c| ((c as f64 + s.t / 3.0).sin() + (r as f64).cos()) * 10.0)
                        .collect()
                })
                .collect();
            let hm = Heatmap::new(grid)
                .gradient(Gradient::Heat)
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Heatmap "));
            f.render_widget(&hm, e);

            help(f, bar, " space pause · t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char(' ') => s.paused = !s.paused,
                _ => {}
            }
            false
        },
        |s| {
            if s.paused {
                return;
            }
            s.t += 1.0;
            let o = s.candles.last().unwrap().close;
            let c = o + s.rng.gauss();
            let candle = Candle::new(o, o.max(c) + s.rng.f64(), o.min(c) - s.rng.f64(), c)
                .volume(1000.0 + s.rng.f64() * 1000.0);
            s.candles.push(candle);
            if s.candles.len() > 80 {
                s.candles.remove(0);
            }
            s.samples.push(s.rng.gauss());
            if s.samples.len() > 800 {
                s.samples.remove(0);
            }
        },
    )
}
