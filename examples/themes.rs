//! Theme gallery: several widgets side by side with the active theme.
//! Keys: t/T next/prev theme · q quit

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
    BarChart, BarGroup, Candle, CandlestickChart, Gauge, Heatmap, LineChart, RadarChart,
    RadarSeries, Series, Sparkline, Theme, Treemap, TreemapItem,
};

struct State {
    theme: Theme,
    candles: Vec<Candle>,
    spark: Vec<f64>,
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(11);
    let mut price = 100.0;
    let candles = (0..40)
        .map(|_| {
            let open = price;
            let close = open + rng.gauss();
            let high = open.max(close) + rng.f64();
            let low = open.min(close) - rng.f64();
            price = close;
            Candle::new(open, high, low, close)
        })
        .collect();
    let spark = (0..80).map(|_| rng.f64() * 10.0).collect();
    run(
        State {
            theme: Theme::default(),
            candles,
            spark,
        },
        Duration::from_millis(200),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(8), Constraint::Length(2)]).areas(f.area());
            let rows = Layout::vertical([Constraint::Ratio(1, 2); 2]).split(main);
            let top = Layout::horizontal([Constraint::Ratio(1, 4); 4]).split(rows[0]);
            let bot = Layout::horizontal([Constraint::Ratio(1, 4); 4]).split(rows[1]);
            let title = |t: &str| Block::bordered().title(format!(" {t} "));

            let line = LineChart::new(vec![
                Series::new("sin").data((0..60).map(|i| (i as f64, (i as f64 / 6.0).sin()))),
                Series::new("cos").data((0..60).map(|i| (i as f64, (i as f64 / 9.0).cos()))),
            ])
            .theme(s.theme.clone())
            .block(title(&format!("Line · {}", s.theme.name)));
            f.render_widget(&line, top[0]);

            let candle = CandlestickChart::new(s.candles.clone())
                .theme(s.theme.clone())
                .block(title("Candles"));
            f.render_widget(&candle, top[1]);

            let bars = BarChart::new(vec![
                BarGroup::new("Q1", [120.0, 80.0]),
                BarGroup::new("Q2", [150.0, 95.0]),
                BarGroup::new("Q3", [90.0, 110.0]),
            ])
            .series_names(["A", "B"])
            .theme(s.theme.clone())
            .block(title("Bars"));
            f.render_widget(&bars, top[2]);

            let spark = Sparkline::new(s.spark.iter().copied())
                .theme(s.theme.clone())
                .block(title("Sparkline"));
            f.render_widget(&spark, top[3]);

            let gauge = Gauge::new(65.0)
                .bands([(50.0, Color::Green), (80.0, Color::Yellow)])
                .label("65%")
                .theme(s.theme.clone())
                .block(title("Gauge"));
            f.render_widget(&gauge, bot[0]);

            let radar = RadarChart::new(["Speed", "Power", "Range", "Armor", "Cost"])
                .series(RadarSeries::new("A", [7.0, 5.0, 8.0, 4.0, 6.0]))
                .series(RadarSeries::new("B", [4.0, 8.0, 5.0, 7.0, 3.0]))
                .max(10.0)
                .theme(s.theme.clone())
                .block(title("Radar"));
            f.render_widget(&radar, bot[1]);

            let data = (0..6)
                .map(|r| (0..10).map(|c| ((r * 10 + c) % 7) as f64).collect())
                .collect();
            let hm = Heatmap::new(data)
                .theme(s.theme.clone())
                .block(title("Heatmap"));
            f.render_widget(&hm, bot[2]);

            let tm = Treemap::new([
                TreemapItem::new("A", 40.0),
                TreemapItem::new("B", 25.0),
                TreemapItem::new("C", 20.0),
                TreemapItem::new("D", 10.0),
                TreemapItem::new("E", 5.0),
            ])
            .theme(s.theme.clone())
            .block(title("Treemap"));
            f.render_widget(&tm, bot[3]);

            help(f, bar, " t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
