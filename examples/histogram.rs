//! Histogram demo.
//! Keys: +/- bins · o orientation · r resample · q quit

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
use tui_chart::{BarOrientation, Histogram};

struct State {
    rng: Rng,
    data: Vec<f64>,
    bins: usize,
    orientation: BarOrientation,
}

fn sample(rng: &mut Rng) -> Vec<f64> {
    (0..2000).map(|_| 50.0 + rng.gauss() * 12.0).collect()
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(3);
    let data = sample(&mut rng);
    run(
        State {
            rng,
            data,
            bins: 12,
            orientation: BarOrientation::Vertical,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
            let h = Histogram::new(s.data.clone())
                .bins(s.bins)
                .color(Color::LightBlue)
                .orientation(s.orientation)
                .show_values(true)
                .block(Block::bordered().title(format!(
                    " Histogram · {} bins · n={} ",
                    s.bins,
                    s.data.len()
                )));
            f.render_widget(&h, main);
            help(f, bar, " +/- bins · o orientation · r resample · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('+') | KeyCode::Char('=') => s.bins = (s.bins + 1).min(60),
                KeyCode::Char('-') => s.bins = s.bins.saturating_sub(1).max(2),
                KeyCode::Char('o') => {
                    s.orientation = match s.orientation {
                        BarOrientation::Vertical => BarOrientation::Horizontal,
                        BarOrientation::Horizontal => BarOrientation::Vertical,
                    }
                }
                KeyCode::Char('r') => s.data = sample(&mut s.rng),
                _ => {}
            }
            false
        },
        |_| {},
    )
}
