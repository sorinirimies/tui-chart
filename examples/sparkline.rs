//! Sparkline demo.
//! Keys: s style · g gradient · space pause · t/T theme · q quit

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
use tui_chart::{Sparkline, SparklineStyle, Theme};

struct State {
    theme: Theme,
    data: Vec<f64>,
    rng: Rng,
    line: bool,
    gradient: bool,
    paused: bool,
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(7);
    let data = (0..200).map(|_| rng.f64() * 10.0).collect();
    let state = State {
        theme: Theme::default(),
        data,
        rng,
        line: false,
        gradient: true,
        paused: false,
    };
    run(
        state,
        Duration::from_millis(100),
        |f, s| {
            let [a, b, bar] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .areas(f.area());
            let style = if s.line {
                SparklineStyle::Line
            } else {
                SparklineStyle::Bar
            };
            let mk = || {
                let sp = Sparkline::new(s.data.iter().copied())
                    .style(style)
                    .highlight_last(Color::White)
                    .highlight_max(Color::Red)
                    .highlight_min(Color::Blue);
                if s.gradient {
                    sp.gradient([Color::Green, Color::Yellow, Color::Red])
                } else {
                    sp
                }
            };
            f.render_widget(
                mk().theme(s.theme.clone())
                    .block(Block::bordered().title(" 1 row ")),
                a,
            );
            f.render_widget(
                mk().theme(s.theme.clone())
                    .block(Block::bordered().title(" tall ")),
                b,
            );
            help(
                f,
                bar,
                " s style · g gradient · space pause · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('s') => s.line = !s.line,
                KeyCode::Char('g') => s.gradient = !s.gradient,
                KeyCode::Char(' ') => s.paused = !s.paused,
                _ => {}
            }
            false
        },
        |s| {
            if !s.paused {
                s.data.remove(0);
                s.data.push(s.rng.f64() * 10.0);
            }
        },
    )
}
