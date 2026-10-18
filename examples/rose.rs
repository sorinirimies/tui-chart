//! Rose (polar area) demo. Keys: a area scale · i hole · s rotate · v values · r resample · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{RoseChart, RoseItem, Theme};

struct State {
    theme: Theme,
    rng: Rng,
    data: Vec<f64>,
    area: bool,
    hole: usize,
    rot: f64,
    values: bool,
}

fn sample(rng: &mut Rng) -> Vec<f64> {
    (0..12).map(|_| 5.0 + rng.f64() * 95.0).collect()
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(17);
    let data = sample(&mut rng);
    run(
        State {
            theme: Theme::default(),
            rng,
            data,
            area: true,
            hole: 0,
            rot: 0.0,
            values: true,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let months = [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ];
            let chart = RoseChart::new(
                s.data
                    .iter()
                    .zip(months)
                    .map(|(v, m)| RoseItem::new(m, v.round())),
            )
            .area_scale(s.area)
            .inner([0.0, 0.3, 0.55][s.hole % 3])
            .start_deg(s.rot)
            .show_values(s.values)
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Monthly rainfall "));
            f.render_widget(&chart, main);
            help(
                f,
                bar,
                " a area scale · i hole · s rotate · v values · r resample · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('a') => s.area = !s.area,
                KeyCode::Char('i') => s.hole += 1,
                KeyCode::Char('s') => s.rot = (s.rot + 15.0) % 360.0,
                KeyCode::Char('v') => s.values = !s.values,
                KeyCode::Char('r') => s.data = sample(&mut s.rng),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
