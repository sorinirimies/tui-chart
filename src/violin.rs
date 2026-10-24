//! Violin plot: mirrored kernel density per group with median and quartiles.

use crate::canvas::{Canvas, Marker};
use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::{darken, dim_text, format_tick, nice_bounds, nice_ticks, pick};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// One violin: a label and its samples.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ViolinGroup {
    /// Group label.
    pub label: String,
    /// Samples; non-finite values are ignored.
    pub samples: Vec<f64>,
    /// Explicit color.
    pub color: Option<Color>,
}

impl ViolinGroup {
    /// Create a group.
    pub fn new(label: impl Into<String>, samples: impl IntoIterator<Item = f64>) -> Self {
        Self {
            label: label.into(),
            samples: samples.into_iter().collect(),
            color: None,
        }
    }

    /// Explicit color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

/// Violin plot widget.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{ViolinChart, ViolinGroup};
///
/// let chart = ViolinChart::new([ViolinGroup::new("a", (0..60).map(|i| (i % 13) as f64))]);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ViolinChart {
    groups: Vec<ViolinGroup>,
    bandwidth: Option<f64>,
    show_box: bool,
    marker: Marker,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    let pos = q * (sorted.len() - 1) as f64;
    let (i, f) = (pos.floor() as usize, pos.fract());
    let j = (i + 1).min(sorted.len() - 1);
    sorted[i] * (1.0 - f) + sorted[j] * f
}

impl ViolinChart {
    /// Create a chart.
    pub fn new(groups: impl IntoIterator<Item = ViolinGroup>) -> Self {
        Self {
            groups: groups.into_iter().collect(),
            show_box: true,
            ..Self::default()
        }
    }

    opt_setter!(
        /// Kernel bandwidth in data units (default: Silverman's rule).
        bandwidth: f64
    );
    setter!(
        /// Draw the inner quartile box and median dot (default on).
        show_box: bool
    );
    setter!(
        /// Dot resolution (default braille; half blocks fill solid but are coarser).
        marker: Marker
    );
    theme_setter!(
        /// Apply a [`Theme`]; explicit group colors win.
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
        let data: Vec<(usize, &ViolinGroup, Vec<f64>)> = self
            .groups
            .iter()
            .enumerate()
            .filter_map(|(i, g)| {
                let mut s: Vec<f64> = g
                    .samples
                    .iter()
                    .copied()
                    .filter(|v| v.is_finite())
                    .collect();
                s.sort_by(f64::total_cmp);
                (!s.is_empty()).then_some((i, g, s))
            })
            .collect();
        if data.is_empty() {
            return;
        }
        let lo = data.iter().map(|d| d.2[0]).fold(f64::MAX, f64::min);
        let hi = data
            .iter()
            .map(|d| *d.2.last().unwrap())
            .fold(f64::MIN, f64::max);
        let (lo, hi) = nice_bounds(lo, hi, 5);
        let span = (hi - lo).max(f64::EPSILON);
        let (dim, text) = dim_text(&self.theme);

        let ticks = nice_ticks(lo, hi, (inner.height as usize / 3).max(2));
        let step = if ticks.len() >= 2 {
            ticks[1] - ticks[0]
        } else {
            1.0
        };
        let labels: Vec<String> = ticks.iter().map(|t| format_tick(*t, step)).collect();
        let gutter = (labels.iter().map(|l| l.chars().count()).max().unwrap_or(1) as u16 + 1)
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
        for (t, l) in ticks.iter().zip(&labels) {
            let f = ((t - lo) / span).clamp(0.0, 1.0);
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
                buf[(x, y)].set_char('┈').set_style(dim);
            }
        }

        let mut cv = Canvas::new(plot.width, plot.height, self.marker);
        let (pw, ph) = (cv.px_w(), cv.px_h());
        let slot = pw / data.len() as i32;
        let ypx = |v: f64| ((1.0 - (v - lo) / span) * (ph - 1) as f64).round() as i32;
        for (k, (i, g, s)) in data.iter().enumerate() {
            let c = g.color.unwrap_or_else(|| pick(&self.theme, *i));
            let fill = darken(c, 0.45);
            let cx = slot * k as i32 + slot / 2;
            let half = ((slot / 2) as f64 * 0.85) as i32;
            let n = s.len();
            let mean = s.iter().sum::<f64>() / n as f64;
            let sd = (s.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
            if crate::interact::recording() {
                let (rx, ry) = self.marker.resolution();
                let (rx, ry) = (i32::from(rx), i32::from(ry));
                let (x_lo, x_hi) = ((cx - half).max(0) / rx, (cx + half).max(0) / rx);
                let (y_lo, y_hi) = (ypx(s[n - 1]).max(0) / ry, ypx(s[0]).max(0) / ry);
                let rect = Rect::new(
                    plot.x + x_lo as u16,
                    plot.y + y_lo as u16,
                    (x_hi - x_lo + 1) as u16,
                    (y_hi - y_lo + 1) as u16,
                )
                .intersection(plot);
                crate::interact::record_point(rect, 0, *i, || {
                    let f = crate::util::format_compact;
                    (
                        g.label.clone(),
                        vec![
                            format!("n {n}  mean {}", f(mean)),
                            format!("median {}", f(quantile(s, 0.5))),
                            format!("range {} .. {}", f(s[0]), f(s[n - 1])),
                        ],
                    )
                });
            }
            let h = self
                .bandwidth
                .filter(|b| *b > 0.0)
                .unwrap_or_else(|| {
                    let bw = 1.06 * sd * (n as f64).powf(-0.2);
                    if bw > 0.0 {
                        bw
                    } else {
                        span / 20.0
                    }
                })
                .max(span * 1e-6);
            // cap work for huge inputs
            let stride = (n / 2000).max(1);
            let pts: Vec<f64> = s.iter().step_by(stride).copied().collect();
            let dens: Vec<f64> = (0..ph)
                .map(|py| {
                    let v = hi - span * py as f64 / (ph - 1).max(1) as f64;
                    pts.iter()
                        .map(|x| (-0.5 * ((v - x) / h).powi(2)).exp())
                        .sum::<f64>()
                })
                .collect();
            let dmax = dens
                .iter()
                .copied()
                .fold(0.0, f64::max)
                .max(f64::MIN_POSITIVE);
            let (smin, smax) = (ypx(s[n - 1]), ypx(s[0]));
            for (py, d) in dens.iter().enumerate() {
                let py = py as i32;
                if py < smin - 2 || py > smax + 2 {
                    continue;
                }
                let hw = (d / dmax * half as f64).round() as i32;
                if hw <= 0 {
                    continue;
                }
                for px in cx - hw..=cx + hw {
                    cv.set(px, py, fill);
                }
                cv.set(cx - hw, py, c);
                cv.set(cx + hw, py, c);
            }
            if self.show_box {
                let (q1, med, q3) = (quantile(s, 0.25), quantile(s, 0.5), quantile(s, 0.75));
                for py in ypx(q3)..=ypx(q1) {
                    cv.set(cx, py, Color::White);
                    cv.set(cx + 1, py, Color::White);
                }
                let m = ypx(med);
                for (dx, dy) in [
                    (0, 0),
                    (1, 0),
                    (-1, 0),
                    (2, 0),
                    (0, 1),
                    (1, 1),
                    (0, -1),
                    (1, -1),
                ] {
                    cv.set(cx + dx, m + dy, c);
                }
            }
        }
        cv.render(buf, plot);
        let slot_w = plot.width / data.len() as u16;
        for (k, (_, g, _)) in data.iter().enumerate() {
            let l: String = g.label.chars().take(slot_w as usize).collect();
            let x =
                plot.x + slot_w * k as u16 + slot_w.saturating_sub(l.chars().count() as u16) / 2;
            put_str(buf, x, inner.bottom() - 1, inner.right(), &l, text);
        }
    }
}

widget_impls!(ViolinChart);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for ViolinChart {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(c: &ViolinChart, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    fn bell(mu: f64, sd: f64) -> Vec<f64> {
        (0..200)
            .map(|i| {
                mu + sd * (((i as f64) * 0.618).fract() * 2.0 - 1.0) * 1.7
                    + sd * ((i % 7) as f64 - 3.0) * 0.2
            })
            .collect()
    }

    fn chart() -> ViolinChart {
        ViolinChart::new([
            ViolinGroup::new("a", bell(5.0, 1.0)).color(Color::Red),
            ViolinGroup::new("b", bell(8.0, 2.0)).color(Color::Blue),
        ])
    }

    #[test]
    fn draws_groups_in_their_colors_with_labels() {
        let b = render(&chart(), 60, 20);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains('a') && t.contains('b'));
        assert!(b.content().iter().any(|c| c.fg == Color::Red));
        assert!(b.content().iter().any(|c| c.fg == Color::Blue));
    }

    #[test]
    fn wider_spread_covers_more_cells() {
        let count = |sd: f64| {
            let c = ViolinChart::new([ViolinGroup::new("x", bell(5.0, sd)).color(Color::Red)])
                .show_box(false);
            render(&c, 40, 20)
                .content()
                .iter()
                .filter(|c| c.symbol() != " " && c.fg != Color::DarkGray && c.fg != Color::Reset)
                .count()
        };
        assert!(count(3.0) > count(0.3));
    }

    #[test]
    fn box_toggle_changes_output() {
        let on = render(&chart(), 60, 20);
        let off = render(&chart().show_box(false), 60, 20);
        assert_ne!(on, off);
        assert!(on.content().iter().any(|c| c.fg == Color::White));
    }

    #[test]
    fn explicit_bandwidth_smooths() {
        let narrow = render(&chart().bandwidth(0.05), 60, 20);
        let wide = render(&chart().bandwidth(3.0), 60, 20);
        assert_ne!(narrow, wide);
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let c = ViolinChart::new([
            ViolinGroup::new("nan", [f64::NAN, f64::INFINITY]),
            ViolinGroup::new("one", [3.0]),
            ViolinGroup::new("same", [2.0, 2.0, 2.0]),
            ViolinGroup::new("big", [-1e300, 1e300]),
        ]);
        for (w, h) in [(0, 0), (1, 1), (8, 4), (3, 40), (200, 60)] {
            render(&c, w, h);
            render(&chart(), w, h);
        }
        render(&ViolinChart::new([]), 40, 10);
    }
}
