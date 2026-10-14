//! Renko / point-and-figure demo.
//! Keys: p toggle renko ↔ point-and-figure · + / - brick size · space pause · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{RenkoChart, RenkoStyle};

struct State {
    prices: Vec<f64>,
    rng: Rng,
    size: f64,
    pf: bool,
    paused: bool,
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(11);
    let mut prices = vec![100.0];
    for _ in 0..300 {
        let l = *prices.last().unwrap();
        prices.push(l + rng.gauss() * 0.8);
    }
    let state = State {
        prices,
        rng,
        size: 1.0,
        pf: false,
        paused: false,
    };
    run(
        state,
        Duration::from_millis(150),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
            let style = if s.pf {
                RenkoStyle::PointAndFigure
            } else {
                RenkoStyle::Bricks
            };
            let chart = RenkoChart::new(&s.prices, s.size).style(style).block(
                Block::bordered().title(format!(" Renko · brick {:.1} · {:?} ", s.size, style)),
            );
            f.render_widget(&chart, main);
            help(f, bar, " p style · +/- brick size · space pause · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('p') => s.pf = !s.pf,
                KeyCode::Char('+') | KeyCode::Char('=') => s.size += 0.5,
                KeyCode::Char('-') => s.size = (s.size - 0.5).max(0.5),
                KeyCode::Char(' ') => s.paused = !s.paused,
                _ => {}
            }
            false
        },
        |s| {
            if s.paused {
                return;
            }
            let l = *s.prices.last().unwrap();
            let n = l + s.rng.gauss() * 0.8;
            s.prices.push(n);
            if s.prices.len() > 600 {
                s.prices.remove(0);
            }
        },
    )
}
