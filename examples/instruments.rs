//! Instrument panels: Oscilloscope, Spectrum, DotMatrix, RadarScope, HexMap.
//! Keys: Tab/→ next page · ← previous · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{jitter, Blip, DotMatrix, HexMap, Oscilloscope, RadarScope, Spectrum, Theme};

struct State {
    theme: Theme,
    page: usize,
    t: f64,
    blips: Vec<Blip>,
    heat: Vec<f64>,
}

const PAGES: [&str; 5] = [
    "Oscilloscope",
    "Spectrum",
    "DotMatrix",
    "RadarScope",
    "HexMap",
];

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(7);
    let blips = (0..7)
        .map(|_| Blip::new(rng.f64(), 0.2 + 0.75 * rng.f64()))
        .collect();
    let heat = (0..42).map(|_| rng.f64()).collect();
    run(
        State {
            theme: Theme::default(),
            page: 0,
            t: 0.0,
            blips,
            heat,
        },
        Duration::from_millis(60),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let block = Block::bordered().title(format!(" {} ", PAGES[s.page]));
            match s.page {
                0 => f.render_widget(
                    Oscilloscope::ecg(3, 360)
                        .phase(s.t * 0.15)
                        .theme(s.theme.clone())
                        .block(block),
                    main,
                ),
                1 => {
                    let v: Vec<f64> = (0..24)
                        .map(|i| jitter(0.55 + 0.4 * ((i as f64) * 0.5).sin().abs(), i, s.t * 3.0))
                        .collect();
                    let peaks: Vec<f64> =
                        (0..24).map(|i| jitter(0.9, i, s.t * 3.0 - 0.4)).collect();
                    f.render_widget(
                        Spectrum::new(v)
                            .peaks(peaks)
                            .theme(s.theme.clone())
                            .block(block),
                        main,
                    );
                }
                2 => {
                    let v: Vec<f64> = (0..40).map(|i| jitter(0.9, i, s.t * 2.0)).collect();
                    f.render_widget(DotMatrix::new(v).theme(s.theme.clone()).block(block), main);
                }
                3 => f.render_widget(
                    RadarScope::new(s.blips.clone())
                        .phase(s.t * 0.25)
                        .theme(s.theme.clone())
                        .block(block),
                    main,
                ),
                _ => f.render_widget(
                    HexMap::new(7, s.heat.clone())
                        .phase(s.t * 0.3)
                        .theme(s.theme.clone())
                        .block(block),
                    main,
                ),
            }
            help(f, bar, " Tab next · ← prev · t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Tab | KeyCode::Right => s.page = (s.page + 1) % PAGES.len(),
                KeyCode::Left => s.page = (s.page + PAGES.len() - 1) % PAGES.len(),
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |s| s.t += 0.06,
    )
}
