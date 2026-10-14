//! BoxPlot demo.
//! Keys: o orientation · r resample · w toggle outliers · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{BoxOrientation, BoxPlot};

struct State {
    rng: Rng,
    data: Vec<Vec<f64>>,
    orientation: BoxOrientation,
    outliers: bool,
}

fn sample(rng: &mut Rng) -> Vec<Vec<f64>> {
    (0..4)
        .map(|g| {
            (0..80)
                .map(|_| 40.0 + g as f64 * 8.0 + rng.gauss() * (6.0 + g as f64 * 2.0))
                .collect()
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(11);
    let data = sample(&mut rng);
    run(
        State {
            rng,
            data,
            orientation: BoxOrientation::Vertical,
            outliers: true,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
            let mut p = BoxPlot::new()
                .orientation(s.orientation)
                .show_outliers(s.outliers)
                .block(Block::bordered().title(" BoxPlot "));
            for (i, d) in s.data.iter().enumerate() {
                p = p.group(format!("G{}", i + 1), d.iter().copied());
            }
            f.render_widget(&p, main);
            help(f, bar, " o orientation · r resample · w outliers · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('o') => {
                    s.orientation = match s.orientation {
                        BoxOrientation::Vertical => BoxOrientation::Horizontal,
                        BoxOrientation::Horizontal => BoxOrientation::Vertical,
                    }
                }
                KeyCode::Char('w') => s.outliers = !s.outliers,
                KeyCode::Char('r') => s.data = sample(&mut s.rng),
                _ => {}
            }
            false
        },
        |_| {},
    )
}
