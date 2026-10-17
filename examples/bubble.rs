//! Bubble chart demo. Keys: k marker · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{BubbleChart, BubbleSeries, Marker, Theme};

struct State {
    theme: Theme,
    marker: Marker,
    data: Vec<Vec<(f64, f64, f64)>>,
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(5);
    let data = (0..3)
        .map(|g| {
            (0..8)
                .map(|_| {
                    (
                        10.0 + g as f64 * 25.0 + rng.f64() * 30.0,
                        20.0 + g as f64 * 15.0 + rng.f64() * 40.0,
                        5.0 + rng.f64() * 95.0,
                    )
                })
                .collect()
        })
        .collect();
    run(
        State {
            theme: Theme::default(),
            marker: Marker::Braille,
            data,
        },
        Duration::from_millis(200),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let names = ["EU", "US", "APAC"];
            let chart = BubbleChart::new(
                s.data
                    .iter()
                    .zip(names)
                    .map(|(d, n)| BubbleSeries::new(n, d.clone())),
            )
            .marker(s.marker)
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Markets: spend vs revenue (size = users) "));
            f.render_widget(&chart, main);
            help(f, bar, " k marker · t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('k') => s.marker = s.marker.next(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
