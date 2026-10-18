//! Lollipop / dumbbell demo. Keys: d dumbbell · s sort · v values · h head · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{LollipopChart, LollipopItem, Theme};

struct State {
    theme: Theme,
    dumbbell: bool,
    sorted: bool,
    values: bool,
    head: usize,
}

const DATA: [(&str, f64, f64); 8] = [
    ("Rust", 82.0, 61.0),
    ("Go", 64.0, 52.0),
    ("Zig", 38.0, 12.0),
    ("Python", 71.0, 74.0),
    ("Kotlin", 55.0, 40.0),
    ("Swift", 47.0, 43.0),
    ("Elixir", 29.0, 18.0),
    ("Haskell", 21.0, 24.0),
];

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            dumbbell: false,
            sorted: true,
            values: true,
            head: 0,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let items = DATA.iter().map(|(n, now, before)| {
                let it = LollipopItem::new(*n, *now);
                if s.dumbbell {
                    it.from(*before)
                } else {
                    it
                }
            });
            let chart = LollipopChart::new(items)
                .sorted(s.sorted)
                .show_values(s.values)
                .head(['●', '◆', '■'][s.head % 3])
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Language satisfaction (now vs last year) "));
            f.render_widget(&chart, main);
            help(
                f,
                bar,
                " d dumbbell · s sort · v values · h head · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('d') => s.dumbbell = !s.dumbbell,
                KeyCode::Char('s') => s.sorted = !s.sorted,
                KeyCode::Char('v') => s.values = !s.values,
                KeyCode::Char('h') => s.head += 1,
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
