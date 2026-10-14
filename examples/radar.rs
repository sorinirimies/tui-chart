//! Radar chart demo.
//! Keys: f fill · k marker · r rings · space animate · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::Color,
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Marker, RadarChart, RadarSeries, Theme};

struct State {
    theme: Theme,
    t: f64,
    fill: bool,
    rings: usize,
    paused: bool,
    marker: Marker,
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            t: 0.0,
            fill: false,
            rings: 4,
            paused: false,
            marker: Marker::Braille,
        },
        Duration::from_millis(80),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
            let a = (0..6).map(|i| 6.0 + 3.0 * (s.t + i as f64).sin());
            let b = (0..6).map(|i| 5.0 + 4.0 * ((s.t * 0.7) + i as f64 * 2.0).cos());
            let c = RadarChart::new(["Speed", "Power", "Range", "Armor", "Stealth", "Cost"])
                .series(RadarSeries::new("A", a).color(Color::Cyan))
                .series(RadarSeries::new("B", b).color(Color::Magenta))
                .max(10.0)
                .rings(s.rings)
                .fill(s.fill)
                .marker(s.marker)
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Radar "));
            f.render_widget(&c, main);
            help(
                f,
                bar,
                " f fill · k marker · r rings · space animate · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('f') => s.fill = !s.fill,
                KeyCode::Char('k') => s.marker = s.marker.next(),
                KeyCode::Char('r') => s.rings = s.rings % 6 + 1,
                KeyCode::Char(' ') => s.paused = !s.paused,
                _ => {}
            }
            false
        },
        |s| {
            if !s.paused {
                s.t += 0.05;
            }
        },
    )
}
