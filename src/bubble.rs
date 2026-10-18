//! Bubble chart: scatter plot where each point has a size.

use crate::canvas::{Canvas, Marker};
use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::{format_tick, nice_bounds, nice_ticks};
use crate::PALETTE;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// A named set of `(x, y, size)` points.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BubbleSeries {
    /// Series name.
    pub name: String,
    /// `(x, y, size)` points; non-finite points are skipped.
    pub points: Vec<(f64, f64, f64)>,
    /// Explicit color.
    pub color: Option<Color>,
}

impl BubbleSeries {
    /// Create a series.
    pub fn new(name: impl Into<String>, points: impl IntoIterator<Item = (f64, f64, f64)>) -> Self {
        Self {
            name: name.into(),
            points: points.into_iter().collect(),
            color: None,
        }
    }

    /// Explicit color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

/// Bubble chart widget. Bubble area is proportional to size.
#[derive(Debug, Clone, Default)]
pub struct BubbleChart {
    series: Vec<BubbleSeries>,
    max_radius: Option<u16>,
    marker: Marker,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl BubbleChart {
    /// Create a bubble chart.
    pub fn new(series: impl IntoIterator<Item = BubbleSeries>) -> Self {
        Self {
            series: series.into_iter().collect(),
            ..Self::default()
        }
    }

    opt_setter!(
        /// Largest bubble radius in dots (default: a fraction of the plot size).
        max_radius: u16
    );

    setter!(
        /// Dot resolution (default braille).
        marker: Marker
    );

    theme_setter!(
        /// Apply a [`Theme`]; explicit series colors win.
        theme
    );

    base_style_setter!(base);

    block_setter!(block, 'static);

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 8 || inner.height < 4 {
            return;
        }
        let pts = || {
            self.series.iter().flat_map(|s| {
                s.points
                    .iter()
                    .filter(|(x, y, z)| x.is_finite() && y.is_finite() && z.is_finite())
            })
        };
        if pts().next().is_none() {
            return;
        }
        let (mut x0, mut x1) = (f64::MAX, f64::MIN);
        let (mut y0, mut y1) = (f64::MAX, f64::MIN);
        let mut zmax = 0f64;
        for &(x, y, z) in pts() {
            x0 = x0.min(x);
            x1 = x1.max(x);
            y0 = y0.min(y);
            y1 = y1.max(y);
            zmax = zmax.max(z.abs());
        }
        let (x0, x1) = nice_bounds(x0, x1, 5);
        let (y0, y1) = nice_bounds(y0, y1, 5);
        let (xs, ys) = ((x1 - x0).max(f64::EPSILON), (y1 - y0).max(f64::EPSILON));
        let dim = Style::default().fg(self.theme.as_ref().map_or(Color::DarkGray, |t| t.grid));
        let text = self
            .theme
            .as_ref()
            .map_or(Style::default(), |t| t.label_style());

        let yt = nice_ticks(y0, y1, (inner.height as usize / 3).max(2));
        let ystep = if yt.len() >= 2 { yt[1] - yt[0] } else { 1.0 };
        let ylabels: Vec<String> = yt.iter().map(|v| format_tick(*v, ystep)).collect();
        let gutter = (ylabels.iter().map(|l| l.chars().count()).max().unwrap_or(1) as u16 + 1)
            .min(inner.width / 2);
        let plot = Rect::new(
            inner.x + gutter,
            inner.y,
            inner.width - gutter,
            inner.height - 1,
        );
        if plot.width < 2 || plot.height < 2 {
            return;
        }
        let mut cv = Canvas::new(plot.width, plot.height, self.marker);
        let (pw, ph) = (cv.px_w(), cv.px_h());
        let rmax = self
            .max_radius
            .map_or(((ph.min(pw / 2)) / 5).max(2), |r| r as i32);
        // Keep whole bubbles inside the plot: map data into the inset area.
        let (ix, iy) = (rmax.min(pw / 3), rmax.min(ph / 3));
        let (aw, ah) = (
            (pw - 1 - 2 * ix).max(1) as f64,
            (ph - 1 - 2 * iy).max(1) as f64,
        );
        for (si, s) in self.series.iter().enumerate() {
            let color = s.color.unwrap_or_else(|| {
                self.theme
                    .as_ref()
                    .map_or(PALETTE[si % PALETTE.len()], |t| t.color(si))
            });
            for &(x, y, z) in &s.points {
                if !(x.is_finite() && y.is_finite() && z.is_finite()) {
                    continue;
                }
                let cx = ix + ((x - x0) / xs * aw).round() as i32;
                let cy = iy + ((1.0 - (y - y0) / ys) * ah).round() as i32;
                let r = if zmax > 0.0 {
                    ((z.abs() / zmax).sqrt() * rmax as f64).round() as i32
                } else {
                    0
                }
                .max(1);
                for dy in -r..=r {
                    for dx in -r..=r {
                        // dots are not square on every marker; scale x by the aspect
                        let (rx, ry) = self.marker.resolution();
                        let ax = dx * ry as i32 / rx as i32;
                        if ax * ax + dy * dy <= r * r {
                            cv.set(cx + dx, cy + dy, color);
                        }
                    }
                }
            }
        }
        for (v, l) in yt.iter().zip(&ylabels) {
            let f = ((v - y0) / ys).clamp(0.0, 1.0);
            let y = plot.bottom() - 1 - (f * (plot.height - 1) as f64).round() as u16;
            let w = l.chars().count() as u16;
            put_str(
                buf,
                inner.x + gutter.saturating_sub(w + 1),
                y,
                plot.x,
                l,
                dim,
            );
            for x in plot.x..plot.right() {
                if buf[(x, y)].symbol() == " " {
                    buf[(x, y)].set_char('┈').set_style(dim);
                }
            }
        }
        cv.render(buf, plot);
        let xt = nice_ticks(x0, x1, (plot.width as usize / 10).max(2));
        let xstep = if xt.len() >= 2 { xt[1] - xt[0] } else { 1.0 };
        for v in xt {
            let f = ((v - x0) / xs).clamp(0.0, 1.0);
            let x = plot.x + (f * (plot.width - 1) as f64).round() as u16;
            let l = format_tick(v, xstep);
            let lx = x.saturating_sub(l.chars().count() as u16 / 2).max(plot.x);
            put_str(buf, lx, inner.bottom() - 1, inner.right(), &l, text);
        }
    }
}

widget_impls!(BubbleChart);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for BubbleChart {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(c: &BubbleChart, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    #[test]
    fn bigger_size_covers_more_cells() {
        let count = |z: f64| {
            let c =
                BubbleChart::new([
                    BubbleSeries::new("a", [(0.0, 0.0, 100.0), (10.0, 10.0, z)]).color(Color::Red)
                ]);
            render(&c, 60, 20)
                .content()
                .iter()
                .filter(|c| c.fg == Color::Red && c.symbol() != " ")
                .count()
        };
        assert!(count(100.0) > count(1.0));
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let c = BubbleChart::new([BubbleSeries::new(
            "a",
            [(f64::NAN, 1.0, 1.0), (1.0, 1.0, 1.0), (1.0, 1.0, -3.0)],
        )]);
        render(&c, 40, 12);
        render(&c, 5, 3);
        render(&c, 0, 0);
        render(&BubbleChart::new([]), 40, 12);
        render(
            &BubbleChart::new([BubbleSeries::new("z", [(0.0, 0.0, 0.0)])]),
            40,
            12,
        );
    }

    #[test]
    fn max_radius_limits_size() {
        let count = |r: u16| {
            let c =
                BubbleChart::new([
                    BubbleSeries::new("a", [(0.0, 0.0, 10.0), (5.0, 5.0, 100.0)]).color(Color::Red),
                ])
                .max_radius(r);
            render(&c, 60, 20)
                .content()
                .iter()
                .filter(|c| c.fg == Color::Red && c.symbol() != " ")
                .count()
        };
        assert!(count(8) > count(2));
    }

    #[test]
    fn multiple_series_and_markers() {
        for m in [Marker::Braille, Marker::HalfBlock, Marker::Dot] {
            let c = BubbleChart::new([
                BubbleSeries::new("a", [(1.0, 1.0, 5.0)]).color(Color::Red),
                BubbleSeries::new("b", [(2.0, 3.0, 9.0)]).color(Color::Blue),
            ])
            .marker(m);
            let b = render(&c, 50, 16);
            assert!(b.content().iter().any(|x| x.fg == Color::Blue));
        }
    }
}
