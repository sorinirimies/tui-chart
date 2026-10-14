//! Histogram demo.
//! Keys: +/- bins · k marker · o orientation · r resample · t/T theme · q quit

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
use tui_chart::{BarOrientation, Histogram, Marker, Theme};

fn cycle(m: Option<Marker>) -> Option<Marker> {
    match m {
        None => Some(Marker::Braille),
        Some(Marker::Sextant) => None,
        Some(m) => Some(m.next()),
    }
}

struct State {
    theme: Theme,
    rng: Rng,
    data: Vec<f64>,
    bins: usize,
    orientation: BarOrientation,
    marker: Option<Marker>,
}

fn sample(rng: &mut Rng) -> Vec<f64> {
    (0..2000).map(|_| 50.0 + rng.gauss() * 12.0).collect()
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(3);
    let data = sample(&mut rng);
    run(
        State {
            theme: Theme::default(),
            rng,
            data,
            bins: 12,
            orientation: BarOrientation::Vertical,
            marker: None,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let mut h = Histogram::new(s.data.clone())
                .bins(s.bins)
                .color(Color::LightBlue)
                .orientation(s.orientation)
                .show_values(true)
                .theme(s.theme.clone())
                .block(Block::bordered().title(format!(
                    " Histogram · {} bins · n={} ",
                    s.bins,
                    s.data.len()
                )));
            if let Some(m) = s.marker {
                h = h.bar_marker(m).marker(m);
            }
            f.render_widget(&h, main);
            help(
                f,
                bar,
                " +/- bins · k marker · o orientation · r resample · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('+') | KeyCode::Char('=') => s.bins = (s.bins + 1).min(60),
                KeyCode::Char('-') => s.bins = s.bins.saturating_sub(1).max(2),
                KeyCode::Char('o') => {
                    s.orientation = match s.orientation {
                        BarOrientation::Vertical => BarOrientation::Horizontal,
                        BarOrientation::Horizontal => BarOrientation::Vertical,
                    }
                }
                KeyCode::Char('k') => s.marker = cycle(s.marker),
                KeyCode::Char('r') => s.data = sample(&mut s.rng),
                _ => {}
            }
            false
        },
        |_| {},
    )
}
