//! RadialBar / RingStack demo.
//! Keys: m mode (bar/stack) · f full sweep · l legend · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{RadialBar, RadialRing, RadialSweep, RingStack, Theme};

struct State {
    theme: Theme,
    stack: bool,
    full: bool,
    legend: bool,
    t: f64,
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            stack: false,
            full: false,
            legend: true,
            t: 0.0,
        },
        Duration::from_millis(80),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let wave = |k: f64| (50.0 + 45.0 * (s.t * 0.7 + k).sin()).clamp(0.0, 100.0);
            let block = Block::bordered().title(" Rings ");
            if s.stack {
                let chart = RingStack::new([
                    ("CPU", wave(0.0)),
                    ("RAM", wave(1.3)),
                    ("Disk", wave(2.6)),
                    ("Net", wave(3.9)),
                ])
                .legend(s.legend)
                .theme(s.theme.clone())
                .block(block);
                f.render_widget(&chart, main);
            } else {
                let chart = RadialBar::new([
                    RadialRing::new("Move", wave(0.0) * 6.0, 600.0),
                    RadialRing::new("Exercise", wave(1.3) * 0.3, 30.0),
                    RadialRing::new("Stand", wave(2.6) * 0.12, 12.0),
                ])
                .sweep(if s.full {
                    RadialSweep::Full
                } else {
                    RadialSweep::Arc
                })
                .legend(s.legend)
                .theme(s.theme.clone())
                .block(block);
                f.render_widget(&chart, main);
            }
            help(f, bar, " m mode · f full sweep · l legend · t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('m') => s.stack = !s.stack,
                KeyCode::Char('f') => s.full = !s.full,
                KeyCode::Char('l') => s.legend = !s.legend,
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |s| s.t += 0.08,
    )
}
