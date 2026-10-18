//! Violin plot demo. Keys: b bandwidth · x inner box · r resample · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Theme, ViolinChart, ViolinGroup};

struct State {
    theme: Theme,
    rng: Rng,
    data: Vec<Vec<f64>>,
    bw: usize,
    boxes: bool,
}

fn sample(rng: &mut Rng) -> Vec<Vec<f64>> {
    let mut a: Vec<f64> = (0..200).map(|_| 30.0 + rng.gauss() * 6.0).collect();
    // bimodal group
    let mut b: Vec<f64> = (0..100).map(|_| 25.0 + rng.gauss() * 4.0).collect();
    b.extend((0..100).map(|_| 55.0 + rng.gauss() * 5.0));
    let c: Vec<f64> = (0..200).map(|_| 45.0 + rng.gauss() * 10.0).collect();
    a.truncate(200);
    vec![a, b, c]
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(8);
    let data = sample(&mut rng);
    run(
        State {
            theme: Theme::default(),
            rng,
            data,
            bw: 0,
            boxes: true,
        },
        Duration::from_millis(500),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let names = ["Unimodal", "Bimodal", "Wide"];
            let mut v = ViolinChart::new(
                s.data
                    .iter()
                    .zip(names)
                    .map(|(d, n)| ViolinGroup::new(n, d.iter().copied())),
            )
            .show_box(s.boxes)
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Latency distributions "));
            if let Some(b) = [None, Some(1.0), Some(3.0), Some(8.0)][s.bw % 4] {
                v = v.bandwidth(b);
            }
            f.render_widget(&v, main);
            help(
                f,
                bar,
                " b bandwidth · x inner box · r resample · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('b') => s.bw += 1,
                KeyCode::Char('x') => s.boxes = !s.boxes,
                KeyCode::Char('r') => s.data = sample(&mut s.rng),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
