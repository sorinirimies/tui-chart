//! Waterfall demo.
//! Keys: c toggle connectors · v toggle values · r randomize · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Theme, Waterfall};

struct State {
    theme: Theme,
    rng: Rng,
    deltas: Vec<f64>,
    connectors: bool,
    values: bool,
}

fn sample(rng: &mut Rng) -> Vec<f64> {
    (0..6).map(|_| (rng.f64() - 0.35) * 100.0).collect()
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(5);
    let deltas = sample(&mut rng);
    run(
        State {
            theme: Theme::default(),
            rng,
            deltas,
            connectors: true,
            values: true,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
            let mut w = Waterfall::new().initial(100.0);
            for (i, d) in s.deltas.iter().enumerate() {
                w = w.delta(format!("Q{}", i + 1), *d);
            }
            let w = w
                .total("Total")
                .connectors(s.connectors)
                .show_values(s.values)
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Waterfall "));
            f.render_widget(&w, main);
            help(
                f,
                bar,
                " c connectors · v values · r randomize · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('c') => s.connectors = !s.connectors,
                KeyCode::Char('v') => s.values = !s.values,
                KeyCode::Char('r') => s.deltas = sample(&mut s.rng),
                _ => {}
            }
            false
        },
        |_| {},
    )
}
