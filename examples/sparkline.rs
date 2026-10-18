//! Sparkline demo: bar, area, line and win/loss styles on a live random walk.
//! Keys: g gradient · m smoothing · k marker · space pause · t/T theme · q quit

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
use tui_chart::{Marker, Sparkline, SparklineStyle, Theme};

struct State {
    theme: Theme,
    data: Vec<f64>,
    rng: Rng,
    gradient: bool,
    smooth: bool,
    paused: bool,
    marker: Marker,
}

fn step(rng: &mut Rng, last: f64) -> f64 {
    (last + rng.gauss() * 1.4).clamp(-30.0, 30.0)
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(7);
    let mut data = vec![0.0];
    for _ in 0..239 {
        let v = step(&mut rng, *data.last().unwrap());
        data.push(v);
    }
    let state = State {
        theme: Theme::default(),
        data,
        rng,
        gradient: true,
        smooth: false,
        paused: false,
        marker: Marker::Braille,
    };
    run(
        state,
        Duration::from_millis(100),
        |f, s| {
            let [a, b, c, d, bar] = Layout::vertical([
                Constraint::Fill(2),
                Constraint::Fill(3),
                Constraint::Fill(2),
                Constraint::Length(3),
                Constraint::Length(2),
            ])
            .areas(f.area());
            let mk = |style, title: &str| {
                let mut sp = Sparkline::new(s.data.iter().copied())
                    .style(style)
                    .marker(s.marker)
                    .smooth(if s.smooth { 4 } else { 1 })
                    .highlight_last(Color::White)
                    .theme(s.theme.clone())
                    .block(Block::bordered().title(format!(" {title} ")));
                if s.gradient {
                    sp = sp.gradient([Color::Red, Color::Yellow, Color::Green]);
                }
                sp
            };
            f.render_widget(mk(SparklineStyle::Bar, "Bar"), a);
            f.render_widget(mk(SparklineStyle::Area, "Area"), b);
            f.render_widget(mk(SparklineStyle::Line, "Line"), c);
            let deltas: Vec<f64> = s.data.windows(2).map(|w| w[1] - w[0]).collect();
            f.render_widget(
                Sparkline::new(deltas)
                    .style(SparklineStyle::WinLoss)
                    .theme(s.theme.clone())
                    .block(Block::bordered().title(" Win / loss (tick-to-tick change) ")),
                d,
            );
            help(
                f,
                bar,
                " g gradient · m smooth · k marker · space pause · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('m') => s.smooth = !s.smooth,
                KeyCode::Char('k') => s.marker = s.marker.next(),
                KeyCode::Char('g') => s.gradient = !s.gradient,
                KeyCode::Char(' ') => s.paused = !s.paused,
                _ => {}
            }
            false
        },
        |s| {
            if !s.paused {
                s.data.remove(0);
                let v = step(&mut s.rng, *s.data.last().unwrap_or(&0.0));
                s.data.push(v);
            }
        },
    )
}
