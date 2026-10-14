//! Order-book depth chart demo.
//! Keys: m mid marker · k dot marker · s spread · space pause · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{DepthChart, Marker, Theme};

fn cycle(m: Option<Marker>) -> Option<Marker> {
    match m {
        None => Some(Marker::Braille),
        Some(Marker::Sextant) => None,
        Some(m) => Some(m.next()),
    }
}

struct State {
    theme: Theme,
    mid: f64,
    rng: Rng,
    show_mid: bool,
    show_spread: bool,
    paused: bool,
    marker: Option<Marker>,
}

fn main() -> anyhow::Result<()> {
    let state = State {
        theme: Theme::default(),
        mid: 100.0,
        rng: Rng::new(7),
        show_mid: true,
        show_spread: true,
        paused: false,
        marker: None,
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
            let mut chart = DepthChart::new(bids, asks)
                .show_mid(s.show_mid)
                .show_spread(s.show_spread)
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Depth chart "));
            if let Some(m) = s.marker {
                chart = chart.marker(m);
            }
            f.render_widget(&chart, main);
            help(
                f,
                bar,
                " m mid marker · k dots · s spread · space pause · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('m') => s.show_mid = !s.show_mid,
                KeyCode::Char('k') => s.marker = cycle(s.marker),
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
