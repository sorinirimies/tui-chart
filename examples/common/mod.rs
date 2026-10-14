//! Shared helpers for the examples: tiny RNG and a terminal event loop.
#![allow(dead_code)]

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};
use std::time::{Duration, Instant};

/// Deterministic xorshift RNG (keeps the examples dependency-free).
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Uniform in [0, 1).
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Approximate standard normal (sum of uniforms).
    pub fn gauss(&mut self) -> f64 {
        (0..12).map(|_| self.f64()).sum::<f64>() - 6.0
    }
}

/// Render a one-line help bar.
pub fn help(f: &mut Frame, area: Rect, text: &str) {
    f.render_widget(
        Paragraph::new(Line::from(text.to_string())).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

/// Run an event loop. `on_key` returns `true` to quit.
pub fn run<S>(
    mut state: S,
    tick: Duration,
    draw: impl Fn(&mut Frame, &S),
    mut on_key: impl FnMut(&mut S, KeyCode) -> bool,
    mut on_tick: impl FnMut(&mut S),
) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let mut last = Instant::now();
    let result = (|| -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| draw(f, &state))?;
            let timeout = tick.saturating_sub(last.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(k) = event::read()? {
                    if k.kind == KeyEventKind::Press && on_key(&mut state, k.code) {
                        return Ok(());
                    }
                }
            }
            if last.elapsed() >= tick {
                on_tick(&mut state);
                last = Instant::now();
            }
        }
    })();
    ratatui::restore();
    result
}
