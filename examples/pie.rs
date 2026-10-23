//! Pie chart demo (re-exported from `tui-piechart`). Keys: r resample · b braille · l legend · q quit

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
use tui_chart::piechart::Resolution;
use tui_chart::{PieChart, PieSlice};

const LANGS: [(&str, Color); 5] = [
    ("Rust", Color::Rgb(222, 165, 132)),
    ("Go", Color::Rgb(0, 173, 216)),
    ("Python", Color::Rgb(255, 212, 59)),
    ("Zig", Color::Rgb(247, 164, 29)),
    ("Other", Color::Rgb(140, 140, 160)),
];

struct State {
    rng: Rng,
    values: Vec<f64>,
    braille: bool,
    legend: bool,
}

fn sample(rng: &mut Rng) -> Vec<f64> {
    LANGS
        .iter()
        .map(|_| (5.0 + rng.f64() * 40.0).round())
        .collect()
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(7);
    let values = sample(&mut rng);
    run(
        State {
            rng,
            values,
            braille: false,
            legend: true,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let slices = LANGS
                .iter()
                .zip(&s.values)
                .map(|((name, color), v)| PieSlice::new(name, *v, *color))
                .collect();
            let chart = PieChart::new(slices)
                .show_legend(s.legend)
                .resolution(if s.braille {
                    Resolution::Braille
                } else {
                    Resolution::Standard
                })
                .block(Block::bordered().title(" Language share "));
            f.render_widget(chart, main);
            help(f, bar, " r resample · b braille · l legend · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('r') => s.values = sample(&mut s.rng),
                KeyCode::Char('b') => s.braille = !s.braille,
                KeyCode::Char('l') => s.legend = !s.legend,
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
