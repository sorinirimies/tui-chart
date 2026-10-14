//! Bar chart demo.
//! Keys: o orientation · k marker · s stacked/grouped · v values · d dataset · t/T theme · q quit

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
use tui_chart::{Axis, BarChart, BarGroup, BarMode, BarOrientation, Marker, Theme};

fn cycle(m: Option<Marker>) -> Option<Marker> {
    match m {
        None => Some(Marker::Braille),
        Some(Marker::Sextant) => None,
        Some(m) => Some(m.next()),
    }
}

struct State {
    theme: Theme,
    orientation: BarOrientation,
    mode: BarMode,
    values: bool,
    set: usize,
    marker: Option<Marker>,
}

fn data(set: usize) -> (Vec<BarGroup>, Vec<&'static str>) {
    if set == 0 {
        (
            vec![
                BarGroup::new("Q1", [120.0, 80.0, 45.0]),
                BarGroup::new("Q2", [150.0, 95.0, 60.0]),
                BarGroup::new("Q3", [90.0, 110.0, 75.0]),
                BarGroup::new("Q4", [200.0, 130.0, 90.0]),
            ],
            vec!["North", "South", "West"],
        )
    } else {
        (
            ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    BarGroup::new(*d, [10.0 + (i * 7 % 13) as f64, 5.0 + (i * 5 % 9) as f64])
                })
                .collect(),
            vec!["Visits", "Signups"],
        )
    }
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            orientation: BarOrientation::Vertical,
            mode: BarMode::Grouped,
            values: true,
            set: 0,
            marker: None,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
            let (groups, names) = data(s.set);
            let mut chart = BarChart::new(groups)
                .series_names(names)
                .colors(vec![Color::Cyan, Color::Yellow, Color::Magenta])
                .orientation(s.orientation)
                .mode(s.mode)
                .show_values(s.values)
                .value_axis(Axis::new().grid(true).title("units"))
                .theme(s.theme.clone())
                .block(
                    Block::bordered()
                        .title(format!(" Bar chart · {:?} · {:?} ", s.orientation, s.mode)),
                );
            if let Some(m) = s.marker {
                chart = chart.marker(m);
            }
            f.render_widget(&chart, main);
            help(
                f,
                bar,
                " o orientation · k marker · s stacked · v values · d dataset · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('o') => {
                    s.orientation = match s.orientation {
                        BarOrientation::Vertical => BarOrientation::Horizontal,
                        BarOrientation::Horizontal => BarOrientation::Vertical,
                    }
                }
                KeyCode::Char('s') => {
                    s.mode = match s.mode {
                        BarMode::Grouped => BarMode::Stacked,
                        BarMode::Stacked => BarMode::Grouped,
                    }
                }
                KeyCode::Char('k') => s.marker = cycle(s.marker),
                KeyCode::Char('v') => s.values = !s.values,
                KeyCode::Char('d') => s.set = (s.set + 1) % 2,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
