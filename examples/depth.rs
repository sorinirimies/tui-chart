//! Order-book depth chart demo.
//! Keys: m mid marker · s spread · space pause · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::DepthChart;

struct State {
    mid: f64,
    rng: Rng,
    show_mid: bool,
    show_spread: bool,
    paused: bool,
}

fn main() -> anyhow::Result<()> {
    let state = State {
        mid: 100.0,
        rng: Rng::new(7),
        show_mid: true,
        show_spread: true,
        paused: false,
    };
    run(
        state,
        Duration::from_millis(250),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
            let bids = (1..=40).map(|i| {
                (
                    s.mid - 0.25 - i as f64 * 0.25,
                    1.0 + (i as f64 * 1.7).sin().abs() * 5.0 + i as f64 * 0.2,
                )
            });
            let asks = (1..=40).map(|i| {
                (
                    s.mid + 0.25 + i as f64 * 0.25,
                    1.0 + (i as f64 * 1.3).cos().abs() * 5.0 + i as f64 * 0.2,
                )
            });
            let chart = DepthChart::new(bids, asks)
                .show_mid(s.show_mid)
                .show_spread(s.show_spread)
                .block(Block::bordered().title(" Depth chart "));
            f.render_widget(&chart, main);
            help(f, bar, " m mid marker · s spread · space pause · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('m') => s.show_mid = !s.show_mid,
                KeyCode::Char('s') => s.show_spread = !s.show_spread,
                KeyCode::Char(' ') => s.paused = !s.paused,
                _ => {}
            }
            false
        },
        |s| {
            if !s.paused {
                s.mid = (s.mid + s.rng.gauss() * 0.1).clamp(90.0, 110.0);
            }
        },
    )
}
