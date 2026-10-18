//! DepthChart: order-book depth chart (cumulative bids left, asks right).

use crate::canvas::{Canvas, Marker};
use crate::theme::{self, Theme};
use crate::util::{darken, format_compact, format_tick};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

const EIGHTHS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Order-book depth chart. Bids are drawn as a cumulative step-area on the
/// left, asks on the right, with a mid-price marker and spread readout.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{DepthChart};
///
/// let chart = DepthChart::new([(99.0, 5.0), (98.0, 9.0)], [(101.0, 4.0), (102.0, 8.0)]);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct DepthChart {
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
    bid_color: Option<Color>,
    ask_color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    marker: Option<Marker>,
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
            bid_color: None,
            ask_color: None,
            theme: None,
            base: None,
            marker: None,
            show_mid: true,
            show_spread: true,
            block: None,
        }
    }

    opt_setter!(
        /// Bid (buy side) color.
        bid_color: Color
    );

    opt_setter!(
        /// Ask (sell side) color.
        ask_color: Color
    );

    theme_setter!(
        /// Apply a [`Theme`]: bull/bear side colors, text colors and background
        /// fill. Explicit [`DepthChart::bid_color`] / [`DepthChart::ask_color`] win.
        theme
    );

    base_style_setter!(base);

    fn bid(&self) -> Color {
        self.bid_color
            .unwrap_or_else(|| self.theme.as_ref().map_or(Color::Green, |t| t.bull))
    }

    fn ask(&self) -> Color {
        self.ask_color
            .unwrap_or_else(|| self.theme.as_ref().map_or(Color::Red, |t| t.bear))
    }

    opt_setter!(
        /// Draw the step-area with dot glyphs (e.g. [`Marker::Braille`] for
        /// 2×4 sub-cell resolution) instead of eighth blocks (the default).
        marker: Marker
    );

    setter!(
        /// Show the mid-price marker line.
        show_mid: bool
    );

    setter!(
        /// Show the spread readout on the top row.
        show_spread: bool
    );

    block_setter!(block, 'static);

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

impl DepthChart {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 2 || inner.height < 3 {
            return;
        }
        let dim_c = self.theme.as_ref().map_or(Color::DarkGray, |t| t.grid);
        let txt_c = self.theme.as_ref().map_or(Color::Gray, |t| t.muted);
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
                (b, self.bid())
            } else {
                (a, self.ask())
            };
            if self.marker.is_some() {
                if mid_col == Some(i) {
                    for r in 0..plot_h {
                        put(buf, inner, x, top + r, "│", Style::default().fg(dim_c));
                    }
                }
                continue;
            }
            let units = if max > 0.0 {
                v / max * plot_h as f64 * 8.0
            } else {
                0.0
            };
            let units = units.round() as usize;
            let top_row = units.div_ceil(8); // rows that hold fill
            for r in 0..plot_h as usize {
                let y = top + plot_h - 1 - r as u16;
                let have = units.saturating_sub(r * 8).min(8);
                // brightest at the edge, fading toward the baseline
                let below = top_row.saturating_sub(r + 1) as f64 / plot_h.max(1) as f64;
                let style = Style::default().fg(darken(color, (below * 0.6).min(0.6)));
                if have >= 8 {
                    put(buf, inner, x, y, "█", style);
                } else if have > 0 {
                    put(buf, inner, x, y, &EIGHTHS[have - 1].to_string(), style);
                } else if mid_col == Some(i) {
                    put(buf, inner, x, y, "│", Style::default().fg(dim_c));
                } else if i % 2 == 0 && plot_h >= 8 && ((r + 1) * 4) % (plot_h as usize) < 4 {
                    // faint quarter grid lines in empty space
                    put(
                        buf,
                        inner,
                        x,
                        y,
                        "·",
                        Style::default().fg(darken(dim_c, 0.3)),
                    );
                }
            }
        }
        if let Some(m) = self.marker.filter(|_| max > 0.0) {
            let mut cv = Canvas::new(inner.width, plot_h, m);
            let (pw, ph) = (cv.px_w(), cv.px_h());
            for px in 0..pw {
                let p = lo + (f64::from(px) + 0.5) / f64::from(pw) * (hi - lo);
                let (b, a) = (self.bid_depth_at(p), self.ask_depth_at(p));
                let (v, color) = if b > 0.0 {
                    (b, self.bid())
                } else {
                    (a, self.ask())
                };
                let n = (v / max * f64::from(ph)).round() as i32;
                for py in (ph - n.clamp(0, ph))..ph {
                    cv.set(px, py, color);
                }
            }
            cv.render(buf, Rect::new(inner.x, top, inner.width, plot_h));
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
                put_str(buf, inner, x, inner.y, &txt, Style::default().fg(txt_c));
            }
        }
        let label_y = inner.bottom() - 1;
        let step = ((hi - lo) / 4.0).max(1e-9);
        let l = format_tick(lo, step);
        let r = format_tick(hi, step);
        let dim = Style::default().fg(txt_c);
        put_str(buf, inner, inner.x, label_y, &l, dim);
        let rx = inner.right().saturating_sub(r.chars().count() as u16);
        put_str(buf, inner, rx.max(inner.x), label_y, &r, dim);
        // quarter price ticks when there is room
        for q in [1.0, 3.0] {
            let t = format_tick(lo + (hi - lo) * q / 4.0, step);
            let n = t.chars().count() as u16;
            let x = inner.x + (inner.width as f64 * q / 4.0) as u16;
            let x = x.saturating_sub(n / 2);
            if x > inner.x + l.chars().count() as u16 + 1 && x + n + 1 < rx {
                put_str(buf, inner, x, label_y, &t, dim);
            }
        }
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

widget_impls!(DepthChart);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for DepthChart {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

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

    #[cfg(feature = "theme")]
    #[test]
    fn theme_colors_and_override() {
        let t = Theme::nord();
        let has = |b: &Buffer, c: Color| b.content().iter().any(|x| x.symbol() == "█" && x.fg == c);
        let b = render(&book().theme(&t), 40, 12);
        assert!(has(&b, t.bull) && has(&b, t.bear));
        assert_eq!(b[(0, 5)].bg, t.background.unwrap());
        let b = render(
            &book()
                .theme(&t)
                .bid_color(Color::Yellow)
                .ask_color(Color::Magenta),
            40,
            12,
        );
        assert!(has(&b, Color::Yellow) && has(&b, Color::Magenta) && !has(&b, t.bull));
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
