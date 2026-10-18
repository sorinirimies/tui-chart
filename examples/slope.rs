//! Slope chart demo. Keys: c trend colors · r randomize · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{SlopeChart, SlopeItem, Theme};

struct State {
    theme: Theme,
    rng: Rng,
    trend: bool,
    data: Vec<(&'static str, f64, f64)>,
}

const NAMES: [&str; 8] = [
    "Rust", "Go", "Zig", "Python", "Kotlin", "Swift", "Elixir", "Java",
];

fn sample(rng: &mut Rng) -> Vec<(&'static str, f64, f64)> {
    NAMES
        .iter()
        .map(|n| {
            let a = 10.0 + rng.f64() * 80.0;
            (
                *n,
                a.round(),
                (a + (rng.f64() - 0.45) * 50.0).clamp(2.0, 99.0).round(),
            )
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(21);
    let data = sample(&mut rng);
    run(
        State {
            theme: Theme::default(),
            rng,
            trend: true,
            data,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let chart = SlopeChart::new(s.data.iter().map(|(n, a, b)| SlopeItem::new(*n, *a, *b)))
                .titles("2022", "2025")
                .trend_colors(s.trend)
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Popularity change "));
            f.render_widget(&chart, main);
            help(f, bar, " c trend colors · r randomize · t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('c') => s.trend = !s.trend,
                KeyCode::Char('r') => s.data = sample(&mut s.rng),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
