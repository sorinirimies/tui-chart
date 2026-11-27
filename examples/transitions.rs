//! TachyonFX transitions over a chart.
//! Keys: 1 boot · 2 dissolve · 3 fade in · 4 fade out · 5 sweep · v next chart · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::Color,
    widgets::Block,
};
use std::{
    cell::RefCell,
    time::{Duration, Instant},
};
use tui_chart::{Theme, TreemapItem, VoronoiChart, VoronoiSite, VoronoiTreemap, anim::Transition};

struct State {
    theme: Theme,
    chart: usize,
    fx: RefCell<Transition>,
    last: RefCell<Instant>,
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            chart: 0,
            fx: RefCell::new(Transition::boot(1500)),
            last: RefCell::new(Instant::now()),
        },
        Duration::from_millis(33),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let block = Block::bordered().title(" TachyonFX transitions ");
            let inner = block.inner(main);
            if s.chart == 0 {
                let items = ["Rust", "Go", "Zig", "C", "Java", "Python"]
                    .iter()
                    .enumerate()
                    .map(|(i, n)| TreemapItem::new(*n, 6.0 + i as f64 * 5.0));
                f.render_widget(
                    VoronoiTreemap::new(items)
                        .theme(s.theme.clone())
                        .block(block),
                    main,
                );
            } else {
                let sites = (0..12).map(|i| {
                    let a = i as f64 * 0.9;
                    VoronoiSite::new(
                        format!("S{i}"),
                        50.0 + a.cos() * (10.0 + i as f64 * 3.0),
                        30.0 + a.sin() * (6.0 + i as f64 * 2.0),
                    )
                    .value(i as f64)
                });
                f.render_widget(
                    VoronoiChart::new(sites)
                        .gradient(tui_chart::Gradient::Viridis)
                        .theme(s.theme.clone())
                        .block(block),
                    main,
                );
            }
            let now = Instant::now();
            let dt = now.duration_since(s.last.replace(now));
            s.fx.borrow_mut().apply(dt, f.buffer_mut(), inner);
            help(
                f,
                bar,
                " 1 boot · 2 dissolve · 3 fade in · 4 fade out · 5 sweep · v next chart · t/T theme · q quit",
            );
        },
        |s, key| {
            let bg = Color::Black;
            let t = match key {
                KeyCode::Char('1') => Some(Transition::boot(1500)),
                KeyCode::Char('2') => Some(Transition::dissolve(1500)),
                KeyCode::Char('3') => Some(Transition::fade_in(bg, 1500)),
                KeyCode::Char('4') => Some(Transition::fade_out(bg, 1500)),
                KeyCode::Char('5') => Some(Transition::sweep_in(bg, 1500)),
                _ => None,
            };
            if let Some(t) = t {
                *s.fx.borrow_mut() = t;
            }
            match key {
                KeyCode::Char('v') => {
                    s.chart ^= 1;
                    *s.fx.borrow_mut() = Transition::boot(1500);
                }
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
