//! Line / step / area / scatter chart demo.
//! Keys: m mode · k marker · g grid · l log · s smooth · e bands · a annotations ·
//!       y 2nd axis · c crosshair (←/→) · space pause · q quit

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
use tui_chart::{Annotation, Axis, Crosshair, LegendPosition, LineChart, LineMode, Marker, Series};

struct State {
    t: f64,
    walk: Vec<f64>,
    rng: Rng,
    mode: LineMode,
    marker: Marker,
    grid: bool,
    paused: bool,
    log: bool,
    smooth: bool,
    bands: bool,
    notes: bool,
    second: bool,
    cursor: Option<f64>,
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
        log: false,
        smooth: false,
        bands: false,
        notes: true,
        second: false,
        cursor: None,
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
            let off = if s.log { 5.0 } else { 0.0 };
            let mut sin_s = Series::new("sin")
                .data(sin.map(|(x, y)| (x, y + off)))
                .color(Color::Cyan);
            if s.bands {
                let band: Vec<(f64, f64)> = (0..120)
                    .map(|i| {
                        let y = ((i as f64 + s.t) / 8.0).sin() * 2.0 + off;
                        (y - 0.6, y + 0.6)
                    })
                    .collect();
                sin_s = sin_s.band(band);
            }
            let cos_s = Series::new("cos")
                .data(cos.map(|(x, y)| (x, y + off)))
                .color(Color::Yellow)
                .y_err((0..120).map(|i| if s.bands && i % 10 == 0 { 0.4 } else { 0.0 }))
                .secondary_axis(s.second);
            let mut chart = LineChart::new(vec![
                sin_s,
                cos_s,
                Series::new("walk")
                    .data(walk.map(|(x, y)| (x, y + off)))
                    .color(Color::Magenta),
            ])
            .y_log(s.log)
            .smooth(s.smooth)
            .crosshair(s.cursor.map(|x| Crosshair::new(x, off)))
            .mode(s.mode)
            .marker(s.marker)
            .legend(LegendPosition::TopRight)
            .x_axis(Axis::new().title("sample").grid(s.grid))
            .y_axis(Axis::new().title("value").grid(s.grid))
            .block(
                Block::bordered().title(format!(" Line chart · {:?} · {:?} ", s.mode, s.marker)),
            );
            if s.notes {
                chart = chart
                    .annotation(Annotation::y_region(off + 1.0, off + 2.0).text("high"))
                    .annotation(Annotation::hline(off).text("zero"))
                    .annotation(Annotation::vline(60.0).text("mid"));
            }
            f.render_widget(&chart, main);
            help(
                f,
                bar,
                " m mode · k marker · g grid · l log · s smooth · e bands · a notes · y 2nd axis · c/←/→ crosshair · q quit",
            );
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
                KeyCode::Char('l') => s.log = !s.log,
                KeyCode::Char('s') => s.smooth = !s.smooth,
                KeyCode::Char('e') => s.bands = !s.bands,
                KeyCode::Char('a') => s.notes = !s.notes,
                KeyCode::Char('y') => s.second = !s.second,
                KeyCode::Char('c') => s.cursor = s.cursor.map_or(Some(60.0), |_| None),
                KeyCode::Left => s.cursor = s.cursor.map(|x| (x - 2.0).max(0.0)),
                KeyCode::Right => s.cursor = s.cursor.map(|x| (x + 2.0).min(119.0)),
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
