//! Stacked area demo. Keys: n normalize · o outline · r resample · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{AreaLayer, StackedAreaChart, Theme};

struct State {
    theme: Theme,
    rng: Rng,
    data: Vec<Vec<f64>>,
    normalize: bool,
    outline: bool,
}

fn sample(rng: &mut Rng) -> Vec<Vec<f64>> {
    (0..4)
        .map(|k| {
            let phase = rng.f64() * 6.0;
            (0..48)
                .map(|i| {
                    20.0 + 12.0 * ((i as f64 / 5.0) + phase + k as f64).sin() + rng.f64() * 4.0
                })
                .collect()
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(4);
    let data = sample(&mut rng);
    run(
        State {
            theme: Theme::default(),
            rng,
            data,
            normalize: false,
            outline: true,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let names = ["web", "api", "batch", "cache"];
            let chart = StackedAreaChart::new(
                s.data
                    .iter()
                    .zip(names)
                    .map(|(d, n)| AreaLayer::new(n, d.iter().copied())),
            )
            .normalize(s.normalize)
            .outline(s.outline)
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Traffic by service "));
            f.render_widget(&chart, main);
            help(
                f,
                bar,
                " n normalize · o outline · r resample · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('n') => s.normalize = !s.normalize,
                KeyCode::Char('o') => s.outline = !s.outline,
                KeyCode::Char('r') => s.data = sample(&mut s.rng),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
