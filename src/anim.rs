//! Animated transitions for any chart via [TachyonFX](https://docs.rs/tachyonfx)
//! (feature `tachyonfx`).
//!
//! Render the chart as usual, then let a [`Transition`] post-process the
//! buffer. Effects only touch the cells inside `area`, so wrap one chart or a
//! whole frame.
//!
//! ```
//! use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
//! use std::time::Duration;
//! use tui_chart::{Sparkline, anim::Transition};
//!
//! let area = Rect::new(0, 0, 30, 6);
//! let mut buf = Buffer::empty(area);
//! let mut t = Transition::boot(600);
//! // every frame:
//! Sparkline::new([1.0, 4.0, 2.0, 8.0]).render(area, &mut buf);
//! t.apply(Duration::from_millis(16), &mut buf, area);
//! ```

use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use std::time::Duration;
use tachyonfx::{Effect, EffectTimer, Interpolation, fx};

/// A running effect plus helpers to feed it frame times.
#[derive(Debug)]
pub struct Transition {
    effect: Effect,
}

impl Transition {
    /// Wrap any TachyonFX [`Effect`].
    pub fn new(effect: Effect) -> Self {
        Self { effect }
    }

    /// Chart materialises from scattered cells (`ms` long).
    pub fn boot(ms: u32) -> Self {
        Self::new(fx::coalesce(EffectTimer::from_ms(
            ms,
            Interpolation::QuadOut,
        )))
    }

    /// Chart disperses into nothing.
    pub fn dissolve(ms: u32) -> Self {
        Self::new(fx::dissolve(EffectTimer::from_ms(
            ms,
            Interpolation::QuadIn,
        )))
    }

    /// Fade in from `from` (e.g. the terminal background).
    pub fn fade_in(from: Color, ms: u32) -> Self {
        Self::new(fx::fade_from(
            from,
            from,
            EffectTimer::from_ms(ms, Interpolation::SineOut),
        ))
    }

    /// Fade out to `to`.
    pub fn fade_out(to: Color, ms: u32) -> Self {
        Self::new(fx::fade_to(
            to,
            to,
            EffectTimer::from_ms(ms, Interpolation::SineIn),
        ))
    }

    /// Sweep a reveal from the left over `from` colour.
    pub fn sweep_in(from: Color, ms: u32) -> Self {
        Self::new(fx::sweep_in(
            tachyonfx::Motion::LeftToRight,
            12,
            0,
            from,
            EffectTimer::from_ms(ms, Interpolation::QuadOut),
        ))
    }

    /// Advance by `delta` and post-process `area` of `buf`. Call after rendering.
    pub fn apply(&mut self, delta: Duration, buf: &mut Buffer, area: Rect) {
        self.effect.process(delta.into(), buf, area);
    }

    /// True once the effect has finished (the chart is then drawn untouched).
    pub fn done(&self) -> bool {
        self.effect.done()
    }

    /// Restart from the beginning.
    pub fn reset(&mut self) {
        self.effect.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn frame(t: &mut Transition, ms: u64) -> Buffer {
        let a = Rect::new(0, 0, 30, 8);
        let mut b = Buffer::empty(a);
        crate::Sparkline::new([1.0, 5.0, 2.0, 8.0, 3.0]).render(a, &mut b);
        t.apply(Duration::from_millis(ms), &mut b, a);
        b
    }

    #[test]
    fn finishes_and_then_leaves_chart_untouched() {
        let a = Rect::new(0, 0, 30, 8);
        let mut plain = Buffer::empty(a);
        crate::Sparkline::new([1.0, 5.0, 2.0, 8.0, 3.0]).render(a, &mut plain);
        let mut t = Transition::boot(200);
        assert!(!t.done());
        frame(&mut t, 50);
        assert!(!t.done());
        let end = frame(&mut t, 500);
        assert!(t.done());
        assert_eq!(end, plain);
    }

    #[test]
    fn dissolve_changes_the_buffer_midway_and_reset_restarts() {
        let mut t = Transition::dissolve(400);
        let mid = frame(&mut t, 200);
        let a = Rect::new(0, 0, 30, 8);
        let mut plain = Buffer::empty(a);
        crate::Sparkline::new([1.0, 5.0, 2.0, 8.0, 3.0]).render(a, &mut plain);
        assert_ne!(mid, plain);
        frame(&mut t, 1000);
        assert!(t.done());
        t.reset();
        assert!(!t.done());
    }

    #[test]
    fn all_presets_run_without_panicking() {
        for mut t in [
            Transition::boot(100),
            Transition::dissolve(100),
            Transition::fade_in(Color::Black, 100),
            Transition::fade_out(Color::Black, 100),
            Transition::sweep_in(Color::Black, 100),
        ] {
            for _ in 0..5 {
                frame(&mut t, 40);
            }
            // empty and out-of-range areas
            let mut b = Buffer::empty(Rect::new(0, 0, 4, 2));
            t.apply(Duration::from_millis(10), &mut b, Rect::new(0, 0, 0, 0));
            t.apply(Duration::from_millis(10), &mut b, Rect::new(0, 0, 4, 2));
        }
    }
}
