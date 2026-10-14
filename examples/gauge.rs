//! Gauge demo.
//! Keys: s style · o orientation · up/down value · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::Color,
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Gauge, GaugeOrientation, GaugeStyle, Theme};

struct State {
    theme: Theme,
    value: f64,
    style: GaugeStyle,
    vertical: bool,
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            value: 55.0,
            style: GaugeStyle::Linear,
            vertical: false,
        },
        Duration::from_millis(200),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
            let o = if s.vertical {
                GaugeOrientation::Vertical
            } else {
                GaugeOrientation::Horizontal
            };
            let g = Gauge::new(s.value)
                .style(s.style)
                .orientation(o)
                .bands([
                    (50.0, Color::Green),
                    (80.0, Color::Yellow),
                    (100.0, Color::Red),
                ])
                .target(70.0)
                .label(format!("{:.0}%", s.value))
                .theme(s.theme.clone())
                .block(Block::bordered().title(format!(" Gauge · {:?} ", s.style)));
            f.render_widget(&g, main);
            help(
                f,
                bar,
                " s style · o orientation · up/down value · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('s') => {
                    s.style = match s.style {
                        GaugeStyle::Linear => GaugeStyle::Bullet,
                        GaugeStyle::Bullet => GaugeStyle::Arc,
                        GaugeStyle::Arc => GaugeStyle::Linear,
                    }
                }
                KeyCode::Char('o') => s.vertical = !s.vertical,
                KeyCode::Up => s.value = (s.value + 5.0).min(100.0),
                KeyCode::Down => s.value = (s.value - 5.0).max(0.0),
                _ => {}
            }
            false
        },
        |_| {},
    )
}
