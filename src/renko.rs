//! RenkoChart: renko bricks (or point-and-figure columns) from a price series.

use crate::theme::{self, Theme};
use crate::util::format_tick;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// A single renko brick.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RenkoBrick {
    /// Price at the brick's start.
    pub open: f64,
    /// Price at the brick's end.
    pub close: f64,
}

impl RenkoBrick {
    /// `true` for an up brick.
    pub fn is_up(&self) -> bool {
        self.close > self.open
    }

    /// Lower edge.
    pub fn low(&self) -> f64 {
        self.open.min(self.close)
    }

    /// Upper edge.
    pub fn high(&self) -> f64 {
        self.open.max(self.close)
    }
}

/// Rendering style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RenkoStyle {
    /// Solid renko bricks.
    #[default]
    Bricks,
    /// Point-and-figure: columns of `X` (up) and `O` (down).
    PointAndFigure,
}

/// Compute renko bricks from closes with a fixed brick size.
///
/// A reversal needs a move of one brick beyond the opposite edge of the last
/// brick. Non-finite closes are skipped; `brick_size <= 0` yields no bricks.
pub fn renko_bricks(closes: &[f64], brick_size: f64) -> Vec<RenkoBrick> {
    let mut out: Vec<RenkoBrick> = Vec::new();
    if !(brick_size.is_finite() && brick_size > 0.0) {
        return out;
    }
    let mut it = closes.iter().copied().filter(|v| v.is_finite());
    let Some(base) = it.next() else {
        return out;
    };
    for p in it {
        // Cap iterations against absurd ratios.
        for _ in 0..100_000 {
            let (up_at, down_at) = match out.last() {
                None => (base + brick_size, base - brick_size),
                Some(b) if b.is_up() => (b.close + brick_size, b.open - brick_size),
                Some(b) => (b.open + brick_size, b.close - brick_size),
            };
            let from = match out.last() {
                None => base,
                Some(b) => b.close,
            };
            if p >= up_at - 1e-12 {
                let open = match out.last() {
                    Some(b) if !b.is_up() => b.open,
                    _ => from,
                };
                out.push(RenkoBrick {
                    open,
                    close: open + brick_size,
                });
            } else if p <= down_at + 1e-12 {
                let open = match out.last() {
                    Some(b) if b.is_up() => b.open,
                    _ => from,
                };
                out.push(RenkoBrick {
                    open,
                    close: open - brick_size,
                });
            } else {
                break;
            }
        }
    }
    out
}

/// Renko / point-and-figure chart widget.
#[derive(Debug, Clone)]
pub struct RenkoChart {
    bricks: Vec<RenkoBrick>,
    brick_size: f64,
    up_color: Option<Color>,
    down_color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    style: RenkoStyle,
    brick_width: u16,
    block: Option<Block<'static>>,
}

impl RenkoChart {
    /// Build from closing prices and a fixed brick size.
    pub fn new(closes: &[f64], brick_size: f64) -> Self {
        Self::from_bricks(renko_bricks(closes, brick_size), brick_size)
    }

    /// Build from precomputed bricks.
    pub fn from_bricks(bricks: Vec<RenkoBrick>, brick_size: f64) -> Self {
        Self {
            bricks,
            brick_size,
            up_color: None,
            down_color: None,
            theme: None,
            base: None,
            style: RenkoStyle::Bricks,
            brick_width: 2,
            block: None,
        }
    }

    opt_setter!(
        /// Up brick / `X` color.
        up_color: Color
    );

    opt_setter!(
        /// Down brick / `O` color.
        down_color: Color
    );

    theme_setter!(
        /// Apply a [`Theme`]: bull/bear brick colors, axis label color and
        /// background fill. Explicit [`RenkoChart::up_color`] /
        /// [`RenkoChart::down_color`] win.
        theme
    );

    base_style_setter!(base);

    fn up(&self) -> Color {
        self.up_color
            .unwrap_or_else(|| self.theme.as_ref().map_or(Color::Green, |t| t.bull))
    }

    fn down(&self) -> Color {
        self.down_color
            .unwrap_or_else(|| self.theme.as_ref().map_or(Color::Red, |t| t.bear))
    }

    setter!(
        /// Rendering style.
        style: RenkoStyle
    );

    /// RenkoBrick width in cells (min 1, bricks style only).
    pub fn brick_width(mut self, w: u16) -> Self {
        self.brick_width = w.max(1);
        self
    }

    block_setter!(block, 'static);

    /// The bricks.
    pub fn bricks(&self) -> &[RenkoBrick] {
        &self.bricks
    }

    /// Point-and-figure columns `(low, high, is_up)` built from the bricks.
    pub fn columns(&self) -> Vec<(f64, f64, bool)> {
        let mut cols: Vec<(f64, f64, bool)> = Vec::new();
        for b in &self.bricks {
            match cols.last_mut() {
                Some(c) if c.2 == b.is_up() => {
                    c.0 = c.0.min(b.low());
                    c.1 = c.1.max(b.high());
                }
                _ => cols.push((b.low(), b.high(), b.is_up())),
            }
        }
        cols
    }
}

impl RenkoChart {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if self.bricks.is_empty() || inner.height == 0 || inner.width < 2 {
            return;
        }
        let pf = self.style == RenkoStyle::PointAndFigure;
        let cols = self.columns();
        let items = if pf { cols.len() } else { self.bricks.len() };
        // Y range
        let lo = self
            .bricks
            .iter()
            .map(|b| b.low())
            .fold(f64::INFINITY, f64::min);
        let hi = self
            .bricks
            .iter()
            .map(|b| b.high())
            .fold(f64::NEG_INFINITY, f64::max);
        let step = if self.brick_size.is_finite() && self.brick_size > 0.0 {
            self.brick_size
        } else {
            ((hi - lo) / 4.0).max(1e-9)
        };
        let l1 = format_tick(lo, step);
        let l2 = format_tick(hi, step);
        let gutter = (l1.chars().count().max(l2.chars().count()) as u16 + 1).min(inner.width / 2);
        let plot = Rect::new(
            inner.x + gutter,
            inner.y,
            inner.width - gutter,
            inner.height,
        );
        if plot.width == 0 {
            return;
        }
        let dim = Style::default().fg(self.theme.as_ref().map_or(Color::DarkGray, |t| t.muted));
        for (txt, y) in [(&l2, plot.y), (&l1, plot.bottom() - 1)] {
            for (i, ch) in txt.chars().take(gutter as usize).enumerate() {
                buf[(inner.x + i as u16, y)]
                    .set_symbol(&ch.to_string())
                    .set_style(dim);
            }
        }
        let span = (hi - lo).max(1e-12);
        let h = plot.height as f64;
        let cw = if pf { 2 } else { self.brick_width };
        let fit = (plot.width / cw).max(1) as usize;
        let skip = items.saturating_sub(fit);
        // price at the centre of row r (r=0 is the top)
        let row_price = |r: u16| hi - (r as f64 + 0.5) / h * span;
        for (n, idx) in (skip..items).enumerate() {
            let x0 = plot.x + n as u16 * cw;
            if x0 >= plot.right() {
                break;
            }
            for r in 0..plot.height {
                let p = row_price(r);
                let y = plot.y + r;
                if pf {
                    let (l, u, up) = cols[idx];
                    let tol = span / h / 2.0;
                    if p >= l - tol && p <= u + tol {
                        let (s, c) = if up {
                            ("X", self.up())
                        } else {
                            ("O", self.down())
                        };
                        buf[(x0, y)].set_symbol(s).set_style(Style::default().fg(c));
                    }
                } else {
                    let b = self.bricks[idx];
                    if p >= b.low() && p <= b.high() {
                        let c = if b.is_up() { self.up() } else { self.down() };
                        for dx in 0..cw {
                            if x0 + dx < plot.right() {
                                buf[(x0 + dx, y)]
                                    .set_symbol("█")
                                    .set_style(Style::default().fg(c));
                            }
                        }
                    }
                }
            }
        }
    }
}

widget_impls!(RenkoChart);

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    #[test]
    fn up_bricks() {
        let b = renko_bricks(&[10.0, 13.5], 1.0);
        assert_eq!(b.len(), 3);
        assert!(b.iter().all(|b| b.is_up()));
        assert_eq!(b[2].close, 13.0);
    }

    #[test]
    fn reversal_needs_extra_brick() {
        // up to 12, then fall to 11.0: not enough (needs <= 10)
        assert_eq!(renko_bricks(&[10.0, 12.0, 11.0], 1.0).len(), 2);
        let b = renko_bricks(&[10.0, 12.0, 10.0], 1.0);
        assert_eq!(b.len(), 3);
        assert!(!b[2].is_up());
        assert_eq!((b[2].open, b[2].close), (11.0, 10.0));
    }

    #[test]
    fn bad_input() {
        assert!(renko_bricks(&[], 1.0).is_empty());
        assert!(renko_bricks(&[1.0, 5.0], 0.0).is_empty());
        assert!(renko_bricks(&[1.0, 5.0], f64::NAN).is_empty());
        assert!(renko_bricks(&[f64::NAN, 1.0], 1.0).is_empty());
    }

    #[test]
    fn columns_group() {
        let c = RenkoChart::new(&[10.0, 13.0, 10.0], 1.0);
        let cols = c.columns();
        assert_eq!(cols.len(), 2);
        assert!(cols[0].2 && !cols[1].2);
    }

    fn draw(c: RenkoChart, w: u16, h: u16) -> Buffer {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        c.render(area, &mut buf);
        buf
    }

    #[test]
    fn renders_bricks_and_pf() {
        let data = [10.0, 14.0, 11.0, 15.0, 9.0];
        let buf = draw(RenkoChart::new(&data, 1.0), 40, 12);
        assert!((0..40).any(|x| (0..12).any(|y| buf[(x, y)].symbol() == "█")));
        let buf = draw(
            RenkoChart::new(&data, 1.0).style(RenkoStyle::PointAndFigure),
            40,
            12,
        );
        let s: String = (0..40)
            .flat_map(|x| (0..12).map(move |y| (x, y)))
            .map(|p| buf[p].symbol().to_string())
            .collect();
        assert!(s.contains('X') && s.contains('O'));
    }

    #[test]
    fn theme_colors_and_override() {
        let t = Theme::tokyo_night();
        let data = [10.0, 14.0, 11.0, 15.0, 9.0];
        let has = |b: &Buffer, c: Color| b.content().iter().any(|x| x.symbol() == "█" && x.fg == c);
        let b = draw(RenkoChart::new(&data, 1.0).theme(&t), 40, 12);
        assert!(has(&b, t.bull) && has(&b, t.bear));
        assert_eq!(b[(39, 0)].bg, t.background.unwrap());
        let b = draw(
            RenkoChart::new(&data, 1.0)
                .theme(&t)
                .up_color(Color::Yellow)
                .down_color(Color::Magenta),
            40,
            12,
        );
        assert!(has(&b, Color::Yellow) && has(&b, Color::Magenta) && !has(&b, t.bull));
    }

    #[test]
    fn tiny_ok() {
        for (w, h) in [(0, 0), (1, 1), (2, 1), (3, 3)] {
            draw(
                RenkoChart::new(&[1.0, 5.0, 2.0], 1.0).block(Block::bordered()),
                w,
                h,
            );
        }
        draw(RenkoChart::new(&[], 1.0), 10, 5);
    }
}
