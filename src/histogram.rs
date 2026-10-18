//! Histogram: bins raw samples and renders them as bars.

use crate::axis::Axis;
use crate::bar::{BarChart, BarGroup, BarOrientation};
use crate::canvas::{Canvas, Marker};
use crate::heatmap::Gradient;
use crate::theme::Theme;
use crate::util::{format_tick, nice_step};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

/// One histogram bin `[start, end)` (last bin includes `end`).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bin {
    /// Inclusive lower edge.
    pub start: f64,
    /// Upper edge.
    pub end: f64,
    /// Number of samples.
    pub count: usize,
}

/// Histogram widget.
#[derive(Debug, Clone)]
pub struct Histogram {
    data: Vec<f64>,
    bins: Option<usize>,
    range: Option<(f64, f64)>,
    color: Option<Color>,
    kde: bool,
    stats: bool,
    kde_bandwidth: Option<f64>,
    gradient: Option<Gradient>,
    theme: Option<Theme>,
    style: Option<Style>,
    orientation: BarOrientation,
    show_values: bool,
    cumulative: bool,
    normalized: bool,
    marker: Marker,
    bar_marker: Option<Marker>,
    block: Option<Block<'static>>,
    value_axis: Axis,
}

impl Histogram {
    /// Create from samples (non-finite values are ignored).
    pub fn new(data: impl IntoIterator<Item = f64>) -> Self {
        Self {
            data: data.into_iter().filter(|v| v.is_finite()).collect(),
            bins: None,
            range: None,
            color: None,
            kde: false,
            stats: false,
            kde_bandwidth: None,
            gradient: None,
            theme: None,
            style: None,
            orientation: BarOrientation::Vertical,
            show_values: false,
            cumulative: false,
            normalized: false,
            marker: Marker::Braille,
            bar_marker: None,
            block: None,
            value_axis: Axis::new().grid(true),
        }
    }

    /// Fixed bin count (default: Sturges' rule).
    pub fn bins(mut self, n: usize) -> Self {
        self.bins = Some(n.max(1));
        self
    }

    /// Fixed sample range.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.range = Some((min, max));
        self
    }

    opt_setter!(
        /// Bar color.
        color: Color
    );

    setter!(
        /// Overlay a Gaussian kernel density estimate curve on the bars
        /// (vertical, non-cumulative histograms only; ignored otherwise).
        /// The curve is scaled to the bar units (counts, or density when normalized).
        kde: bool
    );

    setter!(
        /// Mark the mean (`μ`) and median (`med`) with labelled vertical lines
        /// (vertical, non-cumulative histograms only).
        stats: bool
    );

    opt_setter!(
        /// KDE bandwidth (default: Silverman's rule). Implies nothing unless
        /// [`Histogram::kde`] is on.
        kde_bandwidth: f64
    );

    opt_setter!(
        /// Color bars by height using a gradient (instead of one flat color).
        gradient: Gradient
    );

    /// Apply a [`Theme`] (accent bar color, axes, background). An explicit
    /// [`Histogram::color`] wins.
    #[cfg(feature = "theme")]
    pub fn theme(mut self, theme: impl Into<Theme>) -> Self {
        let t = theme.into();
        self.value_axis.theme_in_place(&t);
        self.theme = Some(t);
        self
    }

    base_style_setter!(style);

    setter!(
        /// Bar direction.
        orientation: BarOrientation
    );

    setter!(
        /// Print counts.
        show_values: bool
    );

    setter!(
        /// Cumulative mode: each bar shows the running total up to and including its bin.
        cumulative: bool
    );

    setter!(
        /// Normalized mode: bars show probability density (area = 1); combined with
        /// [`cumulative`](Self::cumulative) the last bar reaches 1.
        normalized: bool
    );

    opt_setter!(
        /// Draw the bars with dot glyphs (see [`BarChart::marker`]). Default:
        /// eighth blocks.
        bar_marker: Marker
    );

    setter!(
        /// Glyph family of the KDE curve (default [`Marker::Braille`]).
        marker: Marker
    );

    block_setter!(block, 'static);

    /// Count axis.
    pub fn value_axis(mut self, a: Axis) -> Self {
        self.value_axis = a;
        if let Some(t) = &self.theme {
            self.value_axis.theme_in_place(t);
        }
        self
    }

    /// Compute the bins.
    pub fn compute_bins(&self) -> Vec<Bin> {
        if self.data.is_empty() {
            return vec![];
        }
        let n = self
            .bins
            .unwrap_or_else(|| ((self.data.len() as f64).log2().ceil() as usize + 1).clamp(1, 64));
        let (lo, hi) = self.range.unwrap_or_else(|| {
            let lo = self.data.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = self.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            (lo, hi)
        });
        let (lo, hi) = if hi <= lo {
            (lo - 0.5, hi + 0.5)
        } else {
            (lo, hi)
        };
        let width = (hi - lo) / n as f64;
        let mut bins: Vec<Bin> = (0..n)
            .map(|i| Bin {
                start: lo + i as f64 * width,
                end: lo + (i + 1) as f64 * width,
                count: 0,
            })
            .collect();
        for &v in &self.data {
            if v < lo || v > hi {
                continue;
            }
            let i = (((v - lo) / width) as usize).min(n - 1);
            bins[i].count += 1;
        }
        bins
    }

    /// Bar heights per bin, honouring the cumulative / normalized modes.
    pub fn heights(&self) -> Vec<f64> {
        let bins = self.compute_bins();
        let total: usize = bins.iter().map(|b| b.count).sum();
        let mut acc = 0.0;
        bins.iter()
            .map(|b| {
                let mut v = b.count as f64;
                if self.normalized && total > 0 {
                    v /= total as f64;
                    if !self.cumulative {
                        v /= (b.end - b.start).abs().max(f64::MIN_POSITIVE);
                    }
                }
                if self.cumulative {
                    acc += v;
                    acc
                } else {
                    v
                }
            })
            .collect()
    }

    /// Gaussian kernel density estimate sampled at `points` evenly spaced x
    /// values across the bin range, as `(x, density)` pairs ready for a
    /// [`Series`](crate::Series). `bandwidth` of `None` uses Silverman's rule.
    /// Scale by `n * bin_width` to overlay on a count histogram.
    pub fn kde_curve(&self, points: usize, bandwidth: Option<f64>) -> Vec<(f64, f64)> {
        let bins = self.compute_bins();
        let (Some(first), Some(last)) = (bins.first(), bins.last()) else {
            return vec![];
        };
        let (lo, hi) = (first.start, last.end);
        let n = self.data.len() as f64;
        let mean = self.data.iter().sum::<f64>() / n;
        let sd = (self.data.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n).sqrt();
        let h = bandwidth
            .filter(|h| h.is_finite() && *h > 0.0)
            .unwrap_or_else(|| {
                let h = 1.06 * sd * n.powf(-0.2);
                if h.is_finite() && h > 0.0 {
                    h
                } else {
                    (hi - lo) / 10.0
                }
            });
        let points = points.max(2);
        let norm = 1.0 / (n * h * (2.0 * std::f64::consts::PI).sqrt());
        (0..points)
            .map(|i| {
                let x = lo + (hi - lo) * i as f64 / (points - 1) as f64;
                let d: f64 = self
                    .data
                    .iter()
                    .map(|v| (-0.5 * ((x - v) / h).powi(2)).exp())
                    .sum();
                (x, d * norm)
            })
            .collect()
    }

    fn to_bar_chart(&self) -> BarChart {
        let bins = self.compute_bins();
        let step = bins
            .first()
            .map(|b| nice_step((b.end - b.start).abs() / 2.0))
            .unwrap_or(1.0);
        let heights = self.heights();
        let groups: Vec<BarGroup> = bins
            .iter()
            .zip(heights)
            .map(|(b, h)| BarGroup::new(format_tick(b.start, step.min(b.end - b.start)), [h]))
            .collect();
        let mut chart = BarChart::new(groups)
            .colors(vec![self.bar_color()])
            .orientation(self.orientation)
            .show_values(self.show_values)
            .group_gap(0)
            .value_axis(self.value_axis.clone());
        if let Some(b) = self.block.clone() {
            chart = chart.block(b);
        }
        #[cfg(feature = "theme")]
        if let Some(t) = &self.theme {
            chart = chart.theme(t);
        }
        if let Some(s) = self.style {
            chart = chart.base_style(s);
        }
        if let Some(m) = self.bar_marker {
            chart = chart.marker(m);
        }
        if let Some(g) = &self.gradient {
            chart = chart.gradient(g.clone());
        }
        chart
    }

    fn bar_color(&self) -> Color {
        self.color
            .unwrap_or_else(|| self.theme.as_ref().map_or(Color::Cyan, |t| t.accent))
    }

    fn draw_kde(&self, chart: &BarChart, area: Rect, buf: &mut Buffer) {
        let bins = self.compute_bins();
        let Some(first) = bins.first() else { return };
        let Some((plot, lo, hi, bw)) = chart.vertical_geometry(area) else {
            return;
        };
        let span = bins.len() as u16 * bw;
        if span == 0 || span > plot.width || hi <= lo {
            return;
        }
        let bin_w = (first.end - first.start).abs();
        let n = self.data.len() as f64;
        let scale = if self.normalized { 1.0 } else { n * bin_w };
        let curve = self.kde_curve((span as usize * 2).max(8), self.kde_bandwidth);
        let (x0, x1) = (first.start, bins[bins.len() - 1].end);
        let mut canvas = Canvas::new(span, plot.height, self.marker);
        let (pw, ph) = (f64::from(canvas.px_w()), f64::from(canvas.px_h()));
        let color = self.theme.as_ref().map_or(Color::Yellow, |t| t.color(1));
        let mut prev: Option<(i32, i32)> = None;
        for (x, d) in curve {
            let fx = ((x - x0) / (x1 - x0)).clamp(0.0, 1.0);
            let fy = ((d * scale - lo) / (hi - lo)).clamp(0.0, 1.0);
            let p = (
                (fx * (pw - 1.0)).round() as i32,
                ((1.0 - fy) * (ph - 1.0)).round() as i32,
            );
            if let Some(q) = prev {
                canvas.line(q.0, q.1, p.0, p.1, color);
            }
            prev = Some(p);
        }
        canvas.render(buf, Rect::new(plot.x, plot.y, span, plot.height));
    }

    fn draw_stats(&self, chart: &BarChart, area: Rect, buf: &mut Buffer) {
        let bins = self.compute_bins();
        let Some(first) = bins.first() else { return };
        let Some((plot, ..)) = chart.vertical_geometry(area) else {
            return;
        };
        let bw = chart.vertical_geometry(area).map_or(0, |g| g.3);
        let span = bins.len() as u16 * bw;
        let (x0, x1) = (first.start, bins[bins.len() - 1].end);
        if span == 0 || span > plot.width || x1 <= x0 || self.data.is_empty() {
            return;
        }
        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let n = sorted.len();
        let mean = sorted.iter().sum::<f64>() / n as f64;
        let median = if n % 2 == 1 {
            sorted[n / 2]
        } else {
            (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
        };
        let th = self.theme.as_ref();
        let items = [
            (mean, "μ", th.map_or(Color::White, |t| t.foreground), 0u16),
            (
                median,
                "med",
                th.map_or(Color::Yellow, |t| t.color(2)),
                1u16,
            ),
        ];
        for (v, name, color, row) in items {
            let fx = ((v - x0) / (x1 - x0)).clamp(0.0, 1.0);
            let x = plot.x + ((fx * span as f64) as u16).min(span - 1);
            for y in plot.y..plot.bottom() {
                if let Some(c) = buf.cell_mut((x, y)) {
                    c.set_char('┆').set_fg(color);
                }
            }
            let label = format!("{name} {}", format_tick(v, 0.1));
            let lx = if x + 1 + label.chars().count() as u16 <= plot.right() {
                x + 1
            } else {
                x.saturating_sub(label.chars().count() as u16)
            };
            crate::text::put_str(
                buf,
                lx,
                plot.y + row,
                plot.right(),
                &label,
                Style::default().fg(color),
            );
        }
    }
}

impl Histogram {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let chart = self.to_bar_chart();
        (&chart).render(area, buf);
        if self.kde && !self.cumulative && self.orientation == BarOrientation::Vertical {
            self.draw_kde(&chart, area, buf);
        }
        if self.stats && !self.cumulative && self.orientation == BarOrientation::Vertical {
            self.draw_stats(&chart, area, buf);
        }
    }
}

widget_impls!(Histogram);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for Histogram {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    #[test]
    fn stats_draw_mean_and_median_labels() {
        let area = Rect::new(0, 0, 50, 14);
        let mut buf = Buffer::empty(area);
        Histogram::new((0..200).map(|i| (i % 50) as f64))
            .bins(10)
            .stats(true)
            .render(area, &mut buf);
        let all: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(all.contains('┆') && all.contains('μ') && all.contains("med"));
    }

    #[test]
    fn bins_cover_all_samples() {
        let h = Histogram::new((0..100).map(|i| i as f64)).bins(10);
        let b = h.compute_bins();
        assert_eq!(b.len(), 10);
        assert_eq!(b.iter().map(|b| b.count).sum::<usize>(), 100);
        assert!(b.iter().all(|b| b.count == 10));
    }

    #[test]
    fn max_value_in_last_bin() {
        let b = Histogram::new([0.0, 10.0]).bins(2).compute_bins();
        assert_eq!(b[0].count, 1);
        assert_eq!(b[1].count, 1);
    }

    #[test]
    fn sturges_default() {
        let b = Histogram::new((0..64).map(|i| i as f64)).compute_bins();
        assert_eq!(b.len(), 7);
    }

    #[test]
    fn constant_data_ok() {
        let b = Histogram::new([5.0; 4]).bins(3).compute_bins();
        assert_eq!(b.iter().map(|b| b.count).sum::<usize>(), 4);
    }

    #[test]
    fn empty_and_nan() {
        assert!(Histogram::new([f64::NAN]).compute_bins().is_empty());
        let area = Rect::new(0, 0, 20, 8);
        let mut buf = Buffer::empty(area);
        Histogram::new(Vec::<f64>::new()).render(area, &mut buf);
    }

    #[test]
    fn explicit_range_drops_outliers() {
        let b = Histogram::new([1.0, 2.0, 100.0])
            .range(0.0, 10.0)
            .bins(2)
            .compute_bins();
        assert_eq!(b.iter().map(|b| b.count).sum::<usize>(), 2);
    }

    #[test]
    fn renders_bars() {
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        let data: Vec<f64> = (0..200).map(|i| ((i * 37) % 100) as f64).collect();
        Histogram::new(data)
            .bins(8)
            .block(Block::bordered())
            .render(area, &mut buf);
        let t: String = (0..12)
            .flat_map(|y| (0..40).map(move |x| (x, y)))
            .map(|p| buf[p].symbol().to_string())
            .collect();
        assert!(t.contains('█'));
    }

    #[test]
    fn horizontal_renders() {
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        Histogram::new((0..50).map(|i| i as f64))
            .bins(5)
            .orientation(BarOrientation::Horizontal)
            .show_values(true)
            .render(area, &mut buf);
    }

    #[test]
    fn cumulative_ends_at_total() {
        let h = Histogram::new((0..100).map(|i| i as f64)).bins(10);
        let c = h.clone().cumulative(true).heights();
        assert_eq!(*c.last().unwrap(), 100.0);
        assert!(c.windows(2).all(|w| w[0] <= w[1]));
        let n = h.cumulative(true).normalized(true).heights();
        assert!((n.last().unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn density_integrates_to_one() {
        let h = Histogram::new((0..100).map(|i| i as f64)).bins(10);
        let w = h.compute_bins()[0].end - h.compute_bins()[0].start;
        let s: f64 = h.normalized(true).heights().iter().map(|d| d * w).sum();
        assert!((s - 1.0).abs() < 1e-9);
    }

    #[test]
    fn kde_shape_and_degenerate() {
        let h = Histogram::new((0..200).map(|i| ((i * 7) % 50) as f64));
        let k = h.kde_curve(30, None);
        assert_eq!(k.len(), 30);
        assert!(k.iter().all(|p| p.1.is_finite() && p.1 >= 0.0));
        assert!(Histogram::new(Vec::<f64>::new())
            .kde_curve(10, None)
            .is_empty());
        assert!(Histogram::new([3.0; 5])
            .kde_curve(5, None)
            .iter()
            .all(|p| p.1.is_finite()));
    }
}
