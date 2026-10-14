//! Heatmap demo.
//! Keys: g gradient · v values · space pause · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Gradient, Heatmap, Theme};

struct State {
    theme: Theme,
    t: f64,
    gradient: Gradient,
    values: bool,
    paused: bool,
}

fn grid(t: f64) -> Vec<Vec<f64>> {
    (0..7)
        .map(|r| {
            (0..24)
                .map(|c| {
                    let x = c as f64 / 3.0;
                    let y = r as f64 / 1.5;
                    ((x + t).sin() + (y - t / 2.0).cos()) * 50.0 + 50.0
                })
                .collect()
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            t: 0.0,
            gradient: Gradient::Viridis,
            values: false,
            paused: false,
        },
        Duration::from_millis(100),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let hours: Vec<String> = (0..24).map(|h| format!("{h:02}")).collect();
            let hm = Heatmap::new(grid(s.t))
                .row_labels(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"])
                .col_labels(hours)
                .cell_width(3)
                .gradient(s.gradient.clone())
                .show_values(s.values)
                .theme(s.theme.clone())
                .block(Block::bordered().title(format!(" Heatmap · {} ", s.gradient.name())));
            f.render_widget(&hm, main);
            help(
                f,
                bar,
                " g gradient · v values · space pause · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('g') => s.gradient = s.gradient.next(),
                KeyCode::Char('v') => s.values = !s.values,
                KeyCode::Char(' ') => s.paused = !s.paused,
                _ => {}
            }
            false
        },
        |s| {
            if !s.paused {
                s.t += 0.1;
            }
        },
    )
}
