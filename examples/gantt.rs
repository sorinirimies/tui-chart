//! Gantt demo.
//! Keys: ←/→ move today marker · d toggle today · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Gantt, Task, Theme};

struct State {
    theme: Theme,
    today: f64,
    show_today: bool,
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            today: 12.0,
            show_today: true,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let progress =
                |start: f64, end: f64| ((s.today - start) / (end - start)).clamp(0.0, 1.0);
            let spec = [
                ("Research", 0.0, 8.0),
                ("Design", 6.0, 16.0),
                ("Backend", 10.0, 28.0),
                ("Frontend", 14.0, 30.0),
                ("Testing", 26.0, 36.0),
                ("Release", 36.0, 40.0),
            ];
            let mut g = Gantt::new(
                spec.iter()
                    .map(|(n, a, b)| Task::new(*n, *a, *b).progress(progress(*a, *b))),
            )
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Gantt "));
            if s.show_today {
                g = g.today(s.today);
            }
            f.render_widget(&g, main);
            help(f, bar, " ←/→ today · d toggle today · t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Left => s.today = (s.today - 1.0).max(0.0),
                KeyCode::Right => s.today = (s.today + 1.0).min(40.0),
                KeyCode::Char('d') => s.show_today = !s.show_today,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
