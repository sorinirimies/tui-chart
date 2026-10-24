//! Gauge demo.
//! Keys: s style · k marker (arc) · o orientation · up/down value · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Flex, Layout},
    style::Color,
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Gauge, GaugeOrientation, GaugeStyle, Marker, Theme};

struct State {
    theme: Theme,
    value: f64,
    style: GaugeStyle,
    vertical: bool,
    marker: Marker,
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            value: 55.0,
            style: GaugeStyle::Linear,
            vertical: false,
            marker: Marker::Braille,
        },
        Duration::from_millis(200),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let o = if s.vertical {
                GaugeOrientation::Vertical
            } else {
                GaugeOrientation::Horizontal
            };
            let g = Gauge::new(s.value)
                .style(s.style)
                .orientation(o)
                .marker(s.marker)
                .bands([
                    (50.0, Color::Green),
                    (80.0, Color::Yellow),
                    (100.0, Color::Red),
                ])
                .target(70.0)
                .label(format!("{:.0}%", s.value))
                .theme(s.theme.clone())
                .block(Block::bordered().title(format!(" Gauge · {:?} ", s.style)));
            // Linear styles look best as a slim bar; radial and vertical ones want room.
            let (pw, ph) = s.style.preferred_size();
            let area = match (s.style, s.vertical) {
                (GaugeStyle::Linear | GaugeStyle::Bullet | GaugeStyle::Segmented, false) => {
                    let h = main.height.min(5);
                    Layout::vertical([Constraint::Length(h)])
                        .flex(Flex::Center)
                        .areas::<1>(main)[0]
                }
                (GaugeStyle::Linear | GaugeStyle::Bullet | GaugeStyle::Segmented, true) => {
                    let w = main.width.min(11);
                    Layout::horizontal([Constraint::Length(w)])
                        .flex(Flex::Center)
                        .areas::<1>(main)[0]
                }
                (GaugeStyle::Arc, _) => main,
                _ => {
                    let (w, h) = (
                        main.width.min(pw * 2 + 2),
                        main.height.min(ph * 2 + 2).max(3),
                    );
                    let [r] = Layout::vertical([Constraint::Length(h)])
                        .flex(Flex::Center)
                        .areas(main);
                    Layout::horizontal([Constraint::Length(w.max(pw + 2))])
                        .flex(Flex::Center)
                        .areas::<1>(r)[0]
                }
            };
            f.render_widget(&g, area);
            help(
                f,
                bar,
                " s style · k marker (arc) · o orientation · up/down value · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('s') => {
                    let i = GaugeStyle::ALL
                        .iter()
                        .position(|x| *x == s.style)
                        .unwrap_or(0);
                    s.style = GaugeStyle::ALL[(i + 1) % GaugeStyle::ALL.len()];
                }
                KeyCode::Char('k') => s.marker = s.marker.next(),
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
