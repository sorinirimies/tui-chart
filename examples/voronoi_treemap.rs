//! Voronoi treemap demo (cell area ∝ value, groups nested).
//! Keys: o outline (rect/circle/hexagon) · r randomize · s reseed layout · l labels · b borders · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{Rng, help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Theme, TreemapItem, VoronoiOutline, VoronoiTreemap};

struct State {
    theme: Theme,
    rng: Rng,
    weights: Vec<f64>,
    seed: u64,
    outline: VoronoiOutline,
    labels: bool,
    borders: bool,
}

const NAMES: [&str; 12] = [
    "Rust", "Go", "Zig", "C", "C++", "Java", "Python", "Swift", "Kotlin", "Ruby", "Lua", "Elixir",
];
const GROUPS: [&str; 3] = ["Systems", "Apps", "Scripting"];

fn randomize(s: &mut State) {
    s.weights = (0..NAMES.len()).map(|_| 2.0 + s.rng.f64() * 30.0).collect();
}

fn main() -> anyhow::Result<()> {
    let mut state = State {
        theme: Theme::default(),
        rng: Rng::new(5),
        weights: vec![],
        seed: 7,
        outline: VoronoiOutline::Circle,
        labels: true,
        borders: true,
    };
    randomize(&mut state);
    run(
        state,
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let items = GROUPS.iter().enumerate().map(|(g, name)| {
                TreemapItem::new(*name, 0.0)
                    .children((g * 4..g * 4 + 4).map(|i| TreemapItem::new(NAMES[i], s.weights[i])))
            });
            f.render_widget(
                VoronoiTreemap::new(items)
                    .seed(s.seed)
                    .outline(s.outline)
                    .labels(s.labels)
                    .borders(s.borders)
                    .theme(s.theme.clone())
                    .block(Block::bordered().title(" Languages by usage (Voronoi treemap) ")),
                main,
            );
            help(
                f,
                bar,
                " o outline · r randomize · s reseed layout · l labels · b borders · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('o') => {
                    s.outline = match s.outline {
                        VoronoiOutline::Rect => VoronoiOutline::Circle,
                        VoronoiOutline::Circle => VoronoiOutline::Hexagon,
                        VoronoiOutline::Hexagon => VoronoiOutline::Rect,
                    }
                }
                KeyCode::Char('r') => randomize(s),
                KeyCode::Char('s') => s.seed += 1,
                KeyCode::Char('l') => s.labels = !s.labels,
                KeyCode::Char('b') => s.borders = !s.borders,
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
