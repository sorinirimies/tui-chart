//! Histogram: bins raw samples and renders them as bars.

use crate::axis::Axis;
use crate::bar::{BarChart, BarGroup, BarOrientation};
use crate::util::{format_tick, nice_step};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{Block, Widget},
};

/// One histogram bin `[start, end)` (last bin includes `end`).
#[derive(Debug, Clone, Copy, PartialEq)]
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
    color: Color,
    orientation: BarOrientation,
    show_values: bool,
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
            color: Color::Cyan,
            orientation: BarOrientation::Vertical,
            show_values: false,
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

    /// Bar color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Bar direction.
    pub fn orientation(mut self, o: BarOrientation) -> Self {
        self.orientation = o;
        self
    }

    /// Print counts.
    pub fn show_values(mut self, on: bool) -> Self {
        self.show_values = on;
        self
    }

    /// Wrap in a block.
    pub fn block(mut self, b: Block<'static>) -> Self {
        self.block = Some(b);
        self
    }

    /// Count axis.
    pub fn value_axis(mut self, a: Axis) -> Self {
        self.value_axis = a;
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

    fn to_bar_chart(&self) -> BarChart {
        let bins = self.compute_bins();
        let step = bins
            .first()
            .map(|b| nice_step((b.end - b.start).abs() / 2.0))
            .unwrap_or(1.0);
        let groups: Vec<BarGroup> = bins
            .iter()
            .map(|b| {
                BarGroup::new(
                    format_tick(b.start, step.min(b.end - b.start)),
                    [b.count as f64],
                )
            })
            .collect();
        let mut chart = BarChart::new(groups)
            .colors(vec![self.color])
            .orientation(self.orientation)
            .show_values(self.show_values)
            .group_gap(0)
            .value_axis(self.value_axis.clone());
        if let Some(b) = self.block.clone() {
            chart = chart.block(b);
        }
        chart
    }
}

impl Widget for &Histogram {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.to_bar_chart().render(area, buf);
    }
}

impl Widget for Histogram {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
