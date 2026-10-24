//! Cyberpunk look: glow, scanlines, chamfered frames and (feature `shader`)
//! a GPU shader backdrop.
//! Keys: g glow · s scanlines · b shader backdrop · p next preset · t theme · q quit

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
#[cfg(feature = "shader")]
use tui_chart::{ShaderBackdrop, ShaderPreset};

struct State {
    theme: Theme,
    t: f64,
    glow: bool,
    scan: bool,
    #[cfg(feature = "shader")]
    backdrop: std::cell::RefCell<Option<ShaderBackdrop>>,
    #[cfg(feature = "shader")]
    preset: usize,
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
            scan: true,
            #[cfg(feature = "shader")]
            backdrop: std::cell::RefCell::new(None),
            #[cfg(feature = "shader")]
            preset: 0,
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
            let bars: Vec<f64> = (0..20).map(|i| jitter(0.9, i, s.t * 3.0)).collect();
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
                    tui_chart::glow(buf, area, 1, 0.5);
                }
                if scan {
                    tui_chart::scanlines(buf, area, 0.3);
                }
            };
            let buf = f.buffer_mut();
            spec.render(l, buf);
            radar.render(r, buf);
            scope.render(bottom, buf);
            #[cfg(feature = "shader")]
            if let Some(bg) = s.backdrop.borrow_mut().as_mut() {
                bg.apply(main, buf);
            }
            for area in [l, r, bottom] {
                post(buf, area);
            }
            help(
                f,
                bar,
                " g glow · s scanlines · b shader · p preset · t theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('g') => s.glow = !s.glow,
                KeyCode::Char('s') => s.scan = !s.scan,
                KeyCode::Char('t') => s.theme = s.theme.next(),
                #[cfg(feature = "shader")]
                KeyCode::Char('b') => {
                    let on = s.backdrop.get_mut().is_some();
                    *s.backdrop.get_mut() = if on {
                        None
                    } else {
                        ShaderBackdrop::new(ShaderPreset::ALL[s.preset])
                            .ok()
                            .map(|b| b.intensity(0.35))
                    };
                }
                #[cfg(feature = "shader")]
                KeyCode::Char('p') => {
                    s.preset = (s.preset + 1) % ShaderPreset::ALL.len();
                    if s.backdrop.get_mut().is_some() {
                        *s.backdrop.get_mut() = ShaderBackdrop::new(ShaderPreset::ALL[s.preset])
                            .ok()
                            .map(|b| b.intensity(0.35));
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |s| s.t += 0.06,
    )
}
