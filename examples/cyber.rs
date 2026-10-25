//! Cyberpunk look: glow, scanlines and chamfered frames.
//! Keys: g glow · s scanlines · t theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Widget,
};
use std::time::Duration;
use tui_chart::{
    bracket_block, chamfer_block, jitter, Blip, Oscilloscope, RadarScope, Spectrum, Theme,
};

struct State {
    theme: Theme,
    t: f64,
    glow: bool,
    scan: bool,
}

fn main() -> anyhow::Result<()> {
    let blips = [
        Blip::new(0.1, 0.5),
        Blip::new(0.45, 0.8),
        Blip::new(0.8, 0.3),
    ];
    run(
        State {
            theme: Theme::default(),
            t: 0.0,
            glow: true,
            scan: false,
        },
        Duration::from_millis(60),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let [top, bottom] =
                Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .areas(main);
            let [l, r] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(top);
            let bars: Vec<f64> = (0..20).map(|i| jitter(0.9, i, s.t * 1.2)).collect();
            let scope = Oscilloscope::ecg(3, 300)
                .phase(s.t * 0.15)
                .theme(s.theme.clone())
                .block(bracket_block(" ECG "));
            let spec = Spectrum::new(bars)
                .theme(s.theme.clone())
                .block(chamfer_block(" SPECTRUM "));
            let radar = RadarScope::new(blips)
                .phase(s.t * 0.25)
                .theme(s.theme.clone())
                .block(chamfer_block(" RADAR "));
            let (glow, scan) = (s.glow, s.scan);
            let post = |buf: &mut ratatui::buffer::Buffer, area| {
                if glow {
                    tui_chart::glow(buf, area, 1, 0.22);
                }
                if scan {
                    tui_chart::scanlines(buf, area, 0.15);
                }
            };
            f.render_widget(
                ratatui::widgets::Block::new().style(s.theme.surface_style()),
                main,
            );
            let buf = f.buffer_mut();
            spec.render(l, buf);
            radar.render(r, buf);
            scope.render(bottom, buf);
            for area in [l, r, bottom] {
                post(buf, area);
            }
            help(f, bar, " g glow · s scanlines · t theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('g') => s.glow = !s.glow,
                KeyCode::Char('s') => s.scan = !s.scan,
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |s| s.t += 0.06,
    )
}
