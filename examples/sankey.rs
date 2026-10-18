//! Sankey demo. Keys: v values · w node width · r randomize · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Sankey, SankeyLink, Theme};

struct State {
    theme: Theme,
    rng: Rng,
    scale: [f64; 5],
    values: bool,
    width: u16,
}

fn sample(rng: &mut Rng) -> [f64; 5] {
    [
        0.5 + rng.f64(),
        0.5 + rng.f64(),
        0.5 + rng.f64(),
        0.5 + rng.f64(),
        0.5 + rng.f64(),
    ]
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(2);
    let scale = sample(&mut rng);
    run(
        State {
            theme: Theme::default(),
            rng,
            scale,
            values: true,
            width: 2,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let k = &s.scale;
            let links = vec![
                SankeyLink::new("Salary", "Budget", (60.0 * k[0]).round()),
                SankeyLink::new("Freelance", "Budget", (25.0 * k[1]).round()),
                SankeyLink::new("Dividends", "Budget", (10.0 * k[2]).round()),
                SankeyLink::new("Budget", "Housing", (35.0 * k[3]).round()),
                SankeyLink::new("Budget", "Food", (18.0 * k[4]).round()),
                SankeyLink::new("Budget", "Savings", (30.0 * k[0]).round()),
                SankeyLink::new("Budget", "Fun", (12.0 * k[1]).round()),
                SankeyLink::new("Savings", "ETF", (20.0 * k[0]).round()),
                SankeyLink::new("Savings", "Cash", (10.0 * k[2]).round()),
            ];
            let chart = Sankey::new(links)
                .show_values(s.values)
                .node_width(s.width)
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Money flow "));
            f.render_widget(&chart, main);
            help(
                f,
                bar,
                " v values · w node width · r randomize · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('v') => s.values = !s.values,
                KeyCode::Char('w') => s.width = if s.width >= 4 { 1 } else { s.width + 1 },
                KeyCode::Char('r') => s.scale = sample(&mut s.rng),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
