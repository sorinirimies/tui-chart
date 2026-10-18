//! Calendar heatmap demo. Keys: g gradient · m week start · c continuous · l legend · r resample · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{days_from_civil, CalendarHeatmap, Gradient, Theme};

struct State {
    theme: Theme,
    rng: Rng,
    values: Vec<f64>,
    gradient: Option<Gradient>,
    monday: bool,
    continuous: bool,
    legend: bool,
}

fn sample(rng: &mut Rng) -> Vec<f64> {
    (0..182)
        .map(|i| {
            let weekday = i % 7; // 2024-07-01 is a Monday
            let busy = if weekday < 5 { 1.0 } else { 0.25 };
            (rng.f64() * rng.f64() * 12.0 * busy).round()
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(3);
    let values = sample(&mut rng);
    run(
        State {
            theme: Theme::default(),
            rng,
            values,
            gradient: None,
            monday: true,
            continuous: false,
            legend: true,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let mut c = CalendarHeatmap::new(days_from_civil(2024, 7, 1), s.values.iter().copied())
                .monday_first(s.monday)
                .legend(s.legend)
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Contributions · H2 2024 "));
            if let Some(g) = &s.gradient {
                c = c.gradient(g.clone());
            }
            if s.continuous {
                c = c.continuous();
            }
            f.render_widget(&c, main);
            help(
                f,
                bar,
                " g gradient · m week · c smooth · l legend · r resample · t/T · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('g') => {
                    s.gradient = Some(
                        s.gradient
                            .as_ref()
                            .map_or(Gradient::Viridis, Gradient::next),
                    )
                }
                KeyCode::Char('m') => s.monday = !s.monday,
                KeyCode::Char('c') => s.continuous = !s.continuous,
                KeyCode::Char('l') => s.legend = !s.legend,
                KeyCode::Char('r') => s.values = sample(&mut s.rng),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
