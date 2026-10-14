//! DepthChart: order-book depth chart (cumulative bids left, asks right).

use crate::util::{format_compact, format_tick};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

const EIGHTHS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Order-book depth chart. Bids are drawn as a cumulative step-area on the
/// left, asks on the right, with a mid-price marker and spread readout.
#[derive(Debug, Clone)]
pub struct DepthChart {
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
    bid_color: Color,
    ask_color: Color,
    show_mid: bool,
    show_spread: bool,
    block: Option<Block<'static>>,
}

fn clean(levels: impl IntoIterator<Item = (f64, f64)>) -> Vec<(f64, f64)> {
    levels
        .into_iter()
        .filter(|(p, q)| p.is_finite() && q.is_finite() && *q > 0.0)
        .collect()
}

impl DepthChart {
    /// Create from `(price, quantity)` bid and ask levels (any order;
    /// non-finite or non-positive-quantity levels are ignored).
    pub fn new(
        bids: impl IntoIterator<Item = (f64, f64)>,
        asks: impl IntoIterator<Item = (f64, f64)>,
    ) -> Self {
        Self {
            bids: clean(bids),
            asks: clean(asks),
            bid_color: Color::Green,
            ask_color: Color::Red,
            show_mid: true,
            show_spread: true,
            block: None,
        }
    }

    /// Bid (buy side) color.
    pub fn bid_color(mut self, c: Color) -> Self {
        self.bid_color = c;
        self
    }

    /// Ask (sell side) color.
    pub fn ask_color(mut self, c: Color) -> Self {
        self.ask_color = c;
        self
    }

    /// Show the mid-price marker line.
    pub fn show_mid(mut self, on: bool) -> Self {
        self.show_mid = on;
        self
    }

    /// Show the spread readout on the top row.
    pub fn show_spread(mut self, on: bool) -> Self {
        self.show_spread = on;
        self
    }

    /// Wrap in a block.
    pub fn block(mut self, b: Block<'static>) -> Self {
        self.block = Some(b);
        self
    }

    fn best_bid(&self) -> Option<f64> {
        self.bids.iter().map(|l| l.0).reduce(f64::max)
    }

    fn best_ask(&self) -> Option<f64> {
        self.asks.iter().map(|l| l.0).reduce(f64::min)
    }

    /// Mid price, if both sides exist.
    pub fn mid(&self) -> Option<f64> {
        Some((self.best_bid()? + self.best_ask()?) / 2.0)
    }

    /// Spread (best ask - best bid), if both sides exist.
    pub fn spread(&self) -> Option<f64> {
        Some(self.best_ask()? - self.best_bid()?)
    }

    /// Cumulative bid quantity at or above `price` (0 inside/above the spread).
    pub fn bid_depth_at(&self, price: f64) -> f64 {
        match self.best_bid() {
            Some(b) if price <= b => self.bids.iter().filter(|l| l.0 >= price).map(|l| l.1).sum(),
            _ => 0.0,
        }
    }

    /// Cumulative ask quantity at or below `price` (0 inside/below the spread).
    pub fn ask_depth_at(&self, price: f64) -> f64 {
        match self.best_ask() {
            Some(a) if price >= a => self.asks.iter().filter(|l| l.0 <= price).map(|l| l.1).sum(),
            _ => 0.0,
        }
    }

    fn price_range(&self) -> Option<(f64, f64)> {
        let all = self.bids.iter().chain(self.asks.iter()).map(|l| l.0);
        let lo = all.clone().fold(f64::INFINITY, f64::min);
        let hi = all.fold(f64::NEG_INFINITY, f64::max);
        if !lo.is_finite() {
            return None;
        }
        Some(if hi > lo {
            (lo, hi)
        } else {
            (lo - 0.5, hi + 0.5)
        })
    }
}

fn put(buf: &mut Buffer, area: Rect, x: u16, y: u16, s: &str, style: Style) {
    if x < area.right() && y < area.bottom() {
        buf[(x, y)].set_symbol(s).set_style(style);
    }
}

fn put_str(buf: &mut Buffer, area: Rect, x: u16, y: u16, s: &str, style: Style) {
    for (i, ch) in s.chars().enumerate() {
        let xx = x as usize + i;
        if xx >= area.right() as usize {
            break;
        }
        put(buf, area, xx as u16, y, &ch.to_string(), style);
    }
}

impl Widget for &DepthChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = match &self.block {
            Some(b) => {
                let i = b.inner(area);
                b.clone().render(area, buf);
                i
            }
            None => area,
        };
        if inner.width < 2 || inner.height < 3 {
            return;
        }
        let Some((lo, hi)) = self.price_range() else {
            return;
        };
        let w = inner.width as usize;
        let plot_h = inner.height - 2; // top: spread, bottom: price labels
        let top = inner.y + 1;
        let prices: Vec<f64> = (0..w)
            .map(|i| lo + (i as f64 + 0.5) / w as f64 * (hi - lo))
            .collect();
        let depths: Vec<(f64, f64)> = prices
            .iter()
            .map(|&p| (self.bid_depth_at(p), self.ask_depth_at(p)))
            .collect();
        let max = depths
            .iter()
            .flat_map(|d| [d.0, d.1])
            .fold(0.0_f64, f64::max);
        let mid_col = self.mid().filter(|_| self.show_mid).map(|m| {
            (((m - lo) / (hi - lo)) * w as f64)
                .floor()
                .clamp(0.0, w as f64 - 1.0) as usize
        });

        for (i, &(b, a)) in depths.iter().enumerate() {
            let x = inner.x + i as u16;
            let (v, color) = if b > 0.0 {
                (b, self.bid_color)
            } else {
                (a, self.ask_color)
            };
            let units = if max > 0.0 {
                v / max * plot_h as f64 * 8.0
            } else {
                0.0
            };
            let units = units.round() as usize;
            for r in 0..plot_h as usize {
                let y = top + plot_h - 1 - r as u16;
                let have = units.saturating_sub(r * 8).min(8);
                let style = Style::default().fg(color);
                if have >= 8 {
                    put(buf, inner, x, y, "█", style);
                } else if have > 0 {
                    put(buf, inner, x, y, &EIGHTHS[have - 1].to_string(), style);
                } else if mid_col == Some(i) {
                    put(buf, inner, x, y, "│", Style::default().fg(Color::DarkGray));
                }
            }
        }
        if self.show_spread {
            if let (Some(s), Some(m)) = (self.spread(), self.mid()) {
                let pct = if m != 0.0 { s / m * 100.0 } else { 0.0 };
                let txt = format!(
                    "spread {} ({:.2}%)",
                    format_tick(s, (s.abs() / 10.0).max(1e-9)),
                    pct
                );
                let x = inner.x + (inner.width.saturating_sub(txt.chars().count() as u16)) / 2;
                put_str(
                    buf,
                    inner,
                    x,
                    inner.y,
                    &txt,
                    Style::default().fg(Color::Gray),
                );
            }
        }
        let label_y = inner.bottom() - 1;
        let step = ((hi - lo) / 4.0).max(1e-9);
        let l = format_tick(lo, step);
        let r = format_tick(hi, step);
        let dim = Style::default().fg(Color::DarkGray);
        put_str(buf, inner, inner.x, label_y, &l, dim);
        let rx = inner.right().saturating_sub(r.chars().count() as u16);
        put_str(buf, inner, rx.max(inner.x), label_y, &r, dim);
        if let Some(m) = self.mid().filter(|_| self.show_mid) {
            let t = format_tick(m, step);
            let mx = inner.x + (inner.width.saturating_sub(t.chars().count() as u16)) / 2;
            if mx > inner.x + l.chars().count() as u16 && mx + (t.len() as u16) < rx {
                put_str(buf, inner, mx, label_y, &t, dim);
            }
        }
        if max > 0.0 {
            let t = format_compact(max);
            put_str(buf, inner, inner.x, inner.y, &t, dim);
        }
    }
}

impl Widget for DepthChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> DepthChart {
        DepthChart::new(
            [(99.0, 5.0), (98.0, 10.0), (97.0, 20.0)],
            [(101.0, 4.0), (102.0, 8.0), (103.0, 16.0)],
        )
    }

    fn render(c: &DepthChart, w: u16, h: u16) -> Buffer {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        c.render(area, &mut buf);
        buf
    }

    #[test]
    fn mid_and_spread() {
        let c = book();
        assert_eq!(c.mid(), Some(100.0));
        assert_eq!(c.spread(), Some(2.0));
    }

    #[test]
    fn cumulative_depth() {
        let c = book();
        assert_eq!(c.bid_depth_at(97.5), 15.0);
        assert_eq!(c.bid_depth_at(100.0), 0.0);
        assert_eq!(c.ask_depth_at(102.5), 12.0);
        assert_eq!(c.ask_depth_at(100.0), 0.0);
    }

    #[test]
    fn renders_both_colors() {
        let buf = render(&book(), 40, 12);
        let mut g = false;
        let mut r = false;
        for y in 0..12 {
            for x in 0..40 {
                let c = &buf[(x, y)];
                if c.symbol() == "█" {
                    g |= c.fg == Color::Green;
                    r |= c.fg == Color::Red;
                }
            }
        }
        assert!(g && r);
    }

    #[test]
    fn spread_text() {
        let buf = render(&book(), 50, 10);
        let row: String = (0..50).map(|x| buf[(x, 0)].symbol().to_string()).collect();
        assert!(row.contains("spread"));
    }

    #[test]
    fn robust() {
        for (w, h) in [(0, 0), (1, 1), (3, 2), (5, 5)] {
            render(&book(), w, h);
        }
        render(&DepthChart::new([], []), 20, 8);
        render(
            &DepthChart::new([(f64::NAN, 1.0)], [(1.0, f64::NAN)]),
            20,
            8,
        );
        render(&DepthChart::new([(1.0, 1.0)], []), 20, 8);
        render(&DepthChart::new([(1.0, 1.0)], [(1.0, 1.0)]), 20, 8);
    }
}
