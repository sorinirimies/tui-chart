//! Line / step / area / scatter chart demo.
//! Keys: m mode · k marker · g grid · space pause · q quit

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
use tui_chart::{Axis, LegendPosition, LineChart, LineMode, Marker, Series};

struct State {
    t: f64,
    walk: Vec<f64>,
    rng: Rng,
    mode: LineMode,
    marker: Marker,
    grid: bool,
    paused: bool,
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(42);
    let mut walk = vec![0.0];
    for _ in 0..119 {
        let last = *walk.last().unwrap();
        walk.push(last + rng.gauss() * 0.3);
    }
    let state = State {
        t: 0.0,
        walk,
        rng,
        mode: LineMode::Line,
        marker: Marker::Braille,
        grid: true,
        paused: false,
    };
    run(
        state,
        Duration::from_millis(80),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
            let sin = (0..120).map(|i| (i as f64, ((i as f64 + s.t) / 8.0).sin() * 2.0));
            let cos = (0..120).map(|i| (i as f64, ((i as f64 + s.t) / 12.0).cos() * 1.5));
            let walk = s.walk.iter().enumerate().map(|(i, v)| (i as f64, *v));
            let chart = LineChart::new(vec![
                Series::new("sin").data(sin).color(Color::Cyan),
                Series::new("cos").data(cos).color(Color::Yellow),
                Series::new("walk").data(walk).color(Color::Magenta),
            ])
            .mode(s.mode)
            .marker(s.marker)
            .legend(LegendPosition::TopRight)
            .x_axis(Axis::new().title("sample").grid(s.grid))
            .y_axis(Axis::new().title("value").grid(s.grid))
            .block(
                Block::bordered().title(format!(" Line chart · {:?} · {:?} ", s.mode, s.marker)),
            );
            f.render_widget(&chart, main);
            help(f, bar, " m mode · k marker · g grid · space pause · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('m') => {
                    s.mode = match s.mode {
                        LineMode::Line => LineMode::Step,
                        LineMode::Step => LineMode::Area,
                        LineMode::Area => LineMode::Scatter,
                        LineMode::Scatter => LineMode::Line,
                    }
                }
                KeyCode::Char('k') => {
                    s.marker = match s.marker {
                        Marker::Braille => Marker::HalfBlock,
                        Marker::HalfBlock => Marker::Block,
                        Marker::Block => Marker::Dot,
                        Marker::Dot => Marker::Braille,
                    }
                }
                KeyCode::Char('g') => s.grid = !s.grid,
                KeyCode::Char(' ') => s.paused = !s.paused,
                _ => {}
            }
            false
        },
        |s| {
            if s.paused {
                return;
            }
            s.t += 1.0;
            let last = *s.walk.last().unwrap();
            let next = last + s.rng.gauss() * 0.3;
            s.walk.push(next);
            if s.walk.len() > 120 {
                s.walk.remove(0);
            }
        },
    )
}
