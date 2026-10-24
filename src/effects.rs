//! Cyberpunk-style post effects for any widget: [`glow`], [`scanlines`], the
//! [`Fx`] wrapper and cut-corner frames ([`chamfer_block`], [`bracket_block`]).
//!
//! Effects run on the rendered [`Buffer`], so they work with every chart.
//! They need a color terminal (truecolor looks best).
//!
//! ```
//! use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
//! use tui_chart::{chamfer_block, Fx, Sparkline};
//!
//! let area = Rect::new(0, 0, 30, 6);
//! let mut buf = Buffer::empty(area);
//! let chart = Sparkline::new([1.0, 4.0, 2.0, 8.0]).block(chamfer_block(" cpu "));
//! Fx::new(chart).glow(1, 0.35).scanlines(0.25).render(area, &mut buf);
//! ```

use crate::util::{darken, mix};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    symbols::border,
    widgets::{Block, Borders, Widget},
};

fn blank(sym: &str) -> bool {
    sym.chars().all(|c| c == ' ' || c == '\u{2800}')
}

/// Spread each lit cell's color into the background of nearby empty cells.
///
/// `radius` is in cells (cells are about twice as tall as wide, so horizontal
/// reach is shorter); `strength` is `0..=1`. Glyph cells are left untouched.
pub fn glow(buf: &mut Buffer, area: Rect, radius: u16, strength: f64) {
    let area = area.intersection(buf.area);
    if radius == 0 || strength <= 0.0 || area.is_empty() {
        return;
    }
    let mut lit = Vec::new();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let c = &buf[(x, y)];
            if !blank(c.symbol()) && c.fg != Color::Reset {
                lit.push((x, y, c.fg));
            }
        }
    }
    let r = i32::from(radius);
    for (x, y, fg) in lit {
        for dy in -r..=r {
            for dx in -2 * r..=2 * r {
                let dist = (f64::from(dx) * 0.5).hypot(f64::from(dy));
                if dist == 0.0 || dist > f64::from(radius) {
                    continue;
                }
                let (nx, ny) = (i32::from(x) + dx, i32::from(y) + dy);
                if nx < i32::from(area.left())
                    || ny < i32::from(area.top())
                    || nx >= i32::from(area.right())
                    || ny >= i32::from(area.bottom())
                {
                    continue;
                }
                let cell = &mut buf[(nx as u16, ny as u16)];
                if !blank(cell.symbol()) {
                    continue;
                }
                let base = if cell.bg == Color::Reset {
                    Color::Black
                } else {
                    cell.bg
                };
                let a = strength.clamp(0.0, 1.0) * (1.0 - dist / (f64::from(radius) + 1.0)) * 0.5;
                cell.bg = mix(base, fg, a);
            }
        }
    }
}

/// Darken every other row (`strength` `0..=1`), like a CRT scanline mask.
pub fn scanlines(buf: &mut Buffer, area: Rect, strength: f64) {
    let area = area.intersection(buf.area);
    if strength <= 0.0 {
        return;
    }
    for y in area.top()..area.bottom() {
        if (y - area.top()) & 1 == 0 {
            continue;
        }
        for x in area.left()..area.right() {
            let c = &mut buf[(x, y)];
            if c.fg != Color::Reset {
                c.fg = darken(c.fg, strength);
            }
            if c.bg != Color::Reset {
                c.bg = darken(c.bg, strength);
            }
        }
    }
}

/// Wraps a widget and applies post effects after it renders.
#[derive(Debug, Clone)]
pub struct Fx<W> {
    inner: W,
    glow: Option<(u16, f64)>,
    scan: Option<f64>,
}

impl<W> Fx<W> {
    /// Wrap `inner` with no effects yet.
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            glow: None,
            scan: None,
        }
    }

    /// Add a [`glow`] of `radius` cells and `strength` `0..=1`.
    pub fn glow(mut self, radius: u16, strength: f64) -> Self {
        self.glow = Some((radius, strength));
        self
    }

    /// Add [`scanlines`] of `strength` `0..=1`.
    pub fn scanlines(mut self, strength: f64) -> Self {
        self.scan = Some(strength);
        self
    }
}

impl<W: Widget> Widget for Fx<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.inner.render(area, buf);
        if let Some((r, s)) = self.glow {
            glow(buf, area, r, s);
        }
        if let Some(s) = self.scan {
            scanlines(buf, area, s);
        }
    }
}

/// A frame with cut (chamfered) corners and a title.
pub fn chamfer_block(title: &'static str) -> Block<'static> {
    Block::new()
        .borders(Borders::ALL)
        .border_set(border::Set {
            top_left: "╱",
            top_right: "╲",
            bottom_left: "╲",
            bottom_right: "╱",
            ..border::PLAIN
        })
        .title(title)
}

/// A frame with heavy corner brackets and light sides, like a HUD readout.
pub fn bracket_block(title: &'static str) -> Block<'static> {
    Block::new()
        .borders(Borders::ALL)
        .border_set(border::Set {
            top_left: "┏",
            top_right: "┓",
            bottom_left: "┗",
            bottom_right: "┛",
            horizontal_top: "─",
            horizontal_bottom: "─",
            vertical_left: "│",
            vertical_right: "│",
        })
        .title(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf_with_dot() -> (Buffer, Rect) {
        let a = Rect::new(0, 0, 9, 5);
        let mut b = Buffer::empty(a);
        b[(4, 2)].set_char('█').set_fg(Color::Rgb(255, 0, 255));
        (b, a)
    }

    #[test]
    fn glow_tints_neighbours_but_not_far_cells_or_glyphs() {
        let (mut b, a) = buf_with_dot();
        b[(5, 2)].set_char('x');
        glow(&mut b, a, 2, 1.0);
        assert!(matches!(b[(3, 2)].bg, Color::Rgb(r, 0, _) if r > 0));
        assert_eq!(b[(5, 2)].bg, Color::Reset);
        assert_eq!(b[(0, 0)].bg, Color::Reset);
        assert!(matches!(b[(4, 1)].bg, Color::Rgb(..)));
    }

    #[test]
    fn glow_noops_and_edges() {
        let (mut b, a) = buf_with_dot();
        let before = b.clone();
        glow(&mut b, a, 0, 1.0);
        glow(&mut b, a, 2, 0.0);
        assert_eq!(b, before);
        let mut e = Buffer::empty(Rect::new(0, 0, 2, 2));
        e[(0, 0)].set_char('#').set_fg(Color::Red);
        glow(&mut e, Rect::new(0, 0, 50, 50), 3, 1.0);
    }

    #[test]
    fn scanlines_darken_odd_rows_only() {
        let a = Rect::new(0, 0, 2, 3);
        let mut b = Buffer::empty(a);
        for y in 0..3 {
            b[(0, y)].set_char('#').set_fg(Color::Rgb(200, 200, 200));
        }
        scanlines(&mut b, a, 0.5);
        assert_eq!(b[(0, 0)].fg, Color::Rgb(200, 200, 200));
        assert_eq!(b[(0, 1)].fg, Color::Rgb(100, 100, 100));
        assert_eq!(b[(0, 2)].fg, Color::Rgb(200, 200, 200));
    }

    #[test]
    fn fx_applies_after_widget_and_frames_render() {
        let a = Rect::new(0, 0, 12, 5);
        let mut b = Buffer::empty(a);
        let w = crate::Spectrum::new([0.2, 0.9, 0.5]).block(chamfer_block(" fx "));
        Fx::new(w).glow(1, 0.5).scanlines(0.3).render(a, &mut b);
        assert_eq!(b[(0, 0)].symbol(), "╱");
        let mut c = Buffer::empty(a);
        bracket_block("t").render(a, &mut c);
        assert_eq!(c[(0, 0)].symbol(), "┏");
    }
}
