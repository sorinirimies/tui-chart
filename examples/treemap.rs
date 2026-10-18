//! Treemap demo.
//! Keys: g gradient · b borders · v values · r randomize · t/T theme · q quit

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
use tui_chart::{Theme, Treemap, TreemapItem};

struct State {
    theme: Theme,
    rng: Rng,
    weights: Vec<f64>,
    gradient: bool,
    border: bool,
    values: bool,
}

const NAMES: [&str; 8] = ["Rust", "Go", "Zig", "C", "C++", "Java", "Python", "Swift"];

fn randomize(s: &mut State) {
    s.weights = (0..NAMES.len()).map(|_| 1.0 + s.rng.f64() * 30.0).collect();
}

fn main() -> anyhow::Result<()> {
    let mut state = State {
        theme: Theme::default(),
        rng: Rng::new(5),
        weights: vec![],
        gradient: false,
        border: false,
        values: true,
    };
    randomize(&mut state);
    run(
        state,
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let w = &s.weights;
            let items = vec![
                TreemapItem::new("Systems", 0.0)
                    .children((0..4).map(|i| TreemapItem::new(NAMES[i], w[i]))),
                TreemapItem::new("Apps", 0.0)
                    .children((4..8).map(|i| TreemapItem::new(NAMES[i], w[i]))),
            ];
            let mut t = Treemap::new(items).border(s.border).show_values(s.values);
            if s.gradient {
                t = t.gradient(Color::Rgb(30, 60, 160), Color::Rgb(60, 220, 120));
            }
            f.render_widget(
                t.theme(s.theme.clone())
                    .block(Block::bordered().title(" Treemap ")),
                main,
            );
            help(
                f,
                bar,
                " g gradient · b borders · v values · r randomize · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('g') => s.gradient = !s.gradient,
                KeyCode::Char('b') => s.border = !s.border,
                KeyCode::Char('v') => s.values = !s.values,
                KeyCode::Char('r') => randomize(s),
                _ => {}
            }
            false
        },
        |_| {},
    )
}
