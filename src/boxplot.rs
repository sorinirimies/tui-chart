//! BoxPlot: box-and-whisker chart from raw samples or precomputed statistics.

use crate::text::{put_str, trunc};
use crate::theme::{self, Theme};
use crate::util::nice_ticks;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// Direction of the value axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BoxOrientation {
    /// Values run bottom to top, groups left to right.
    #[default]
    Vertical,
    /// Values run left to right, groups top to bottom.
    Horizontal,
}

/// Five-number summary plus outliers.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoxStats {
    /// Lower whisker end.
    pub min: f64,
    /// First quartile.
    pub q1: f64,
    /// Median.
    pub median: f64,
    /// Third quartile.
    pub q3: f64,
    /// Upper whisker end.
    pub max: f64,
    /// Points outside the whiskers.
    pub outliers: Vec<f64>,
    /// Arithmetic mean, when known (set by [`BoxStats::from_samples`]).
    #[cfg_attr(feature = "serde", serde(default))]
    pub mean: Option<f64>,
    /// Number of samples behind the statistics (0 = unknown); drives the notch width.
    #[cfg_attr(feature = "serde", serde(default))]
    pub n: usize,
    /// Raw samples (at most 300, evenly thinned) for jittered points.
    #[cfg_attr(feature = "serde", serde(default))]
    pub points: Vec<f64>,
}

impl BoxStats {
    /// Precomputed statistics (no outliers).
    pub fn new(min: f64, q1: f64, median: f64, q3: f64, max: f64) -> Self {
        Self {
            min,
            q1,
            median,
            q3,
            max,
            outliers: vec![],
            mean: None,
            n: 0,
            points: vec![],
        }
    }

    /// Set the mean.
    pub fn mean(mut self, m: f64) -> Self {
        self.mean = Some(m).filter(|m| m.is_finite());
        self
    }

    /// Set the sample count (used for notches).
    pub fn count(mut self, n: usize) -> Self {
        self.n = n;
        self
    }

    /// Raw samples to show as jittered points (non-finite values dropped).
    pub fn points(mut self, p: impl IntoIterator<Item = f64>) -> Self {
        self.points = p.into_iter().filter(|v| v.is_finite()).collect();
        self
    }

    /// Add outliers.
    pub fn outliers(mut self, o: impl IntoIterator<Item = f64>) -> Self {
        self.outliers = o.into_iter().filter(|v| v.is_finite()).collect();
        self
    }

    /// Compute from samples with whiskers at `k` × IQR (non-finite values ignored).
    /// Returns `None` when there are no finite samples.
    pub fn from_samples(samples: &[f64], k: f64) -> Option<Self> {
        let mut s: Vec<f64> = samples.iter().copied().filter(|v| v.is_finite()).collect();
        if s.is_empty() {
            return None;
        }
        s.sort_by(|a, b| a.total_cmp(b));
        let q = |p: f64| {
            let pos = p * (s.len() - 1) as f64;
            let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
            s[lo] + (s[hi] - s[lo]) * (pos - lo as f64)
        };
        let (q1, med, q3) = (q(0.25), q(0.5), q(0.75));
        let k = if k.is_finite() && k >= 0.0 { k } else { 1.5 };
        let iqr = q3 - q1;
        let (lo_f, hi_f) = (q1 - k * iqr, q3 + k * iqr);
        let mut inside = s.iter().filter(|v| **v >= lo_f && **v <= hi_f);
        let min = inside.clone().next().copied().unwrap_or(q1);
        let max = inside.next_back().copied().unwrap_or(q3);
        let outliers = s
            .iter()
            .copied()
            .filter(|v| *v < lo_f || *v > hi_f)
            .collect();
        let stride = (s.len() / 300).max(1);
        Some(Self {
            min,
            q1,
            median: med,
            q3,
            max,
            outliers,
            mean: Some(s.iter().sum::<f64>() / s.len() as f64),
            n: s.len(),
            points: s.iter().step_by(stride).copied().collect(),
        })
    }

    fn valid(&self) -> bool {
        [self.min, self.q1, self.median, self.q3, self.max]
            .iter()
            .all(|v| v.is_finite())
    }
}

#[derive(Debug, Clone)]
struct Group {
    label: String,
    stats: Option<BoxStats>,
}

/// Box-and-whisker chart widget.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{BoxPlot};
///
/// let chart = BoxPlot::new().group("a", (1..=20).map(|x| x as f64)).group("b", (5..=30).map(|x| x as f64));
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct BoxPlot {
    groups: Vec<Group>,
    colors: Option<Vec<Color>>,
    theme: Option<Theme>,
    base: Option<Style>,
    orientation: BoxOrientation,
    whisker_iqr: f64,
    show_outliers: bool,
    show_mean: bool,
    notched: bool,
    jitter: bool,
    range: Option<(f64, f64)>,
    block: Option<Block<'static>>,
}

impl Default for BoxPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxPlot {
    /// Empty chart.
    pub fn new() -> Self {
        Self {
            groups: vec![],
            colors: None,
            theme: None,
            base: None,
            orientation: BoxOrientation::Vertical,
            whisker_iqr: 1.5,
            show_outliers: true,
            show_mean: true,
            notched: false,
            jitter: false,
            range: None,
            block: None,
        }
    }

    /// Add a group computed from raw samples (uses the current whisker factor,
    /// so call [`BoxPlot::whisker_iqr`] first if you change it).
    pub fn group(
        mut self,
        label: impl Into<String>,
        samples: impl IntoIterator<Item = f64>,
    ) -> Self {
        let v: Vec<f64> = samples.into_iter().collect();
        self.groups.push(Group {
            label: label.into(),
            stats: BoxStats::from_samples(&v, self.whisker_iqr),
        });
        self
    }

    /// Add a group from precomputed statistics.
    pub fn stats(mut self, label: impl Into<String>, stats: BoxStats) -> Self {
        self.groups.push(Group {
            label: label.into(),
            stats: Some(stats),
        });
        self
    }

    setter!(
        /// Whisker length as a multiple of the IQR (default 1.5).
        whisker_iqr: f64
    );

    /// Per-group colors (cycled).
    pub fn colors(mut self, c: Vec<Color>) -> Self {
        if !c.is_empty() {
            self.colors = Some(c);
        }
        self
    }

    theme_setter!(
        /// Apply a [`Theme`]: palette group colors, outlier (bear) color, axis
        /// colors and background fill. Explicit [`BoxPlot::colors`] win.
        theme
    );

    base_style_setter!(base);

    fn group_color(&self, i: usize) -> Color {
        if let Some(c) = &self.colors {
            return c[i % c.len()];
        }
        match &self.theme {
            Some(t) => t.color(i),
            None => [
                Color::Cyan,
                Color::Magenta,
                Color::Yellow,
                Color::Green,
                Color::LightRed,
                Color::LightBlue,
            ][i % 6],
        }
    }

    setter!(
        /// Orientation.
        orientation: BoxOrientation
    );

    setter!(
        /// Draw outliers (default on).
        show_outliers: bool
    );

    setter!(
        /// Mark the mean with a `◆` when it is known (default on).
        show_mean: bool
    );

    setter!(
        /// Notch the box around the median's ~95 % confidence interval
        /// (`1.58 × IQR / √n`); needs the sample count (default off).
        notched: bool
    );

    setter!(
        /// Overlay the raw samples as deterministically jittered dots
        /// (only for groups built from samples; default off).
        jitter: bool
    );

    /// Fixed value range.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.range = Some((min, max));
        self
    }

    block_setter!(block, 'static);

    fn value_range(&self) -> (f64, f64) {
        if let Some((a, b)) = self.range {
            if a.is_finite() && b.is_finite() && b > a {
                return (a, b);
            }
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for s in self
            .groups
            .iter()
            .filter_map(|g| g.stats.as_ref())
            .filter(|s| s.valid())
        {
            lo = lo.min(s.min);
            hi = hi.max(s.max);
            if self.show_outliers {
                for &o in &s.outliers {
                    lo = lo.min(o);
                    hi = hi.max(o);
                }
            }
        }
        if !lo.is_finite() {
            return (0.0, 1.0);
        }
        if hi <= lo {
            return (lo - 1.0, hi + 1.0);
        }
        let pad = (hi - lo) * 0.05;
        (lo - pad, hi + pad)
    }
}

impl BoxPlot {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 4 || inner.height < 3 || self.groups.is_empty() {
            return;
        }
        let vert = self.orientation == BoxOrientation::Vertical;
        let (lo, hi) = self.value_range();
        let dim = Style::default().fg(self.theme.as_ref().map_or(Color::DarkGray, |t| t.grid));
        let text = self
            .theme
            .as_ref()
            .map_or(Style::default(), |t| t.label_style());
        let ticks = |len: u16| nice_ticks(lo, hi, (len as usize / 4).max(2));
        let step = |t: &[f64]| if t.len() >= 2 { t[1] - t[0] } else { 1.0 };

        // Layout: gutter (left) and footer (bottom) are label areas.
        let (gutter, plot);
        if vert {
            let t = ticks(inner.height);
            let w = t
                .iter()
                .map(|v| crate::util::format_tick(*v, step(&t)).chars().count())
                .max()
                .unwrap_or(1) as u16;
            gutter = (w + 1).min(inner.width / 2);
            plot = Rect::new(
                inner.x + gutter,
                inner.y,
                inner.width - gutter,
                inner.height - 1,
            );
        } else {
            let w = self
                .groups
                .iter()
                .map(|g| g.label.chars().count())
                .max()
                .unwrap_or(0) as u16;
            gutter = (w + 1).min(inner.width / 3);
            plot = Rect::new(
                inner.x + gutter,
                inner.y,
                inner.width - gutter,
                inner.height - 1,
            );
        }
        if plot.width == 0 || plot.height == 0 {
            return;
        }
        let (along_len, cross_len) = if vert {
            (plot.height, plot.width)
        } else {
            (plot.width, plot.height)
        };
        let to_a = |v: f64| -> i32 {
            let f = ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
            (f * (along_len - 1) as f64).round() as i32
        };
        let cell = |a: i32, c: i32| -> Option<(u16, u16)> {
            if a < 0 || c < 0 || a >= along_len as i32 || c >= cross_len as i32 {
                return None;
            }
            Some(if vert {
                (plot.x + c as u16, plot.y + plot.height - 1 - a as u16)
            } else {
                (plot.x + a as u16, plot.y + c as u16)
            })
        };
        let put = |buf: &mut Buffer, a: i32, c: i32, ch: char, st: Style| {
            if let Some((x, y)) = cell(a, c) {
                buf[(x, y)].set_char(ch).set_style(st);
            }
        };

        // Value axis labels + faint grid.
        let t = ticks(along_len);
        let st = step(&t);
        for v in &t {
            let a = to_a(*v);
            let label = crate::util::format_tick(*v, st);
            if vert {
                if let Some((_, y)) = cell(a, 0) {
                    let lw = label.chars().count() as u16;
                    put_str(
                        buf,
                        inner.x + gutter.saturating_sub(lw + 1),
                        y,
                        plot.x,
                        &label,
                        dim,
                    );
                }
            } else if let Some((x, _)) = cell(a, 0) {
                put_str(buf, x, inner.bottom() - 1, inner.right(), &label, dim);
            }
            for c in 0..cross_len as i32 {
                if let Some((x, y)) = cell(a, c) {
                    if buf[(x, y)].symbol() == " " {
                        buf[(x, y)].set_char('·').set_style(dim);
                    }
                }
            }
        }

        // Groups.
        let n = self.groups.len();
        let slot = (cross_len as f64 / n as f64).max(0.0);
        for (i, g) in self.groups.iter().enumerate() {
            let c0 = (i as f64 * slot).floor() as i32;
            let c1 = (((i + 1) as f64 * slot).floor() as i32).max(c0 + 1);
            let span = c1 - c0;
            let bw = if span >= 5 { span - 2 } else { span.max(1) };
            let bw = if vert { bw.min(15) } else { bw.min(5) };
            let left = c0 + (span - bw) / 2;
            let mid = left + bw / 2;
            let color = self.group_color(i);
            let s = Style::default().fg(color);

            // label
            if vert {
                let w = span.max(0) as usize;
                let l = trunc(&g.label, w);
                let off = (w.saturating_sub(l.chars().count()) / 2) as u16;
                put_str(
                    buf,
                    plot.x + c0 as u16 + off,
                    inner.bottom() - 1,
                    inner.right(),
                    &l,
                    text,
                );
            } else if let Some((_, y)) = cell(0, mid) {
                put_str(
                    buf,
                    inner.x,
                    y,
                    plot.x,
                    &trunc(&g.label, gutter.saturating_sub(1) as usize),
                    text,
                );
            }

            let Some(st) = g.stats.as_ref().filter(|s| s.valid()) else {
                continue;
            };
            let (amin, aq1, amed, aq3, amax) = (
                to_a(st.min),
                to_a(st.q1),
                to_a(st.median),
                to_a(st.q3),
                to_a(st.max),
            );
            let (wl, cp, ml) = if vert {
                ('│', '─', '━')
            } else {
                ('─', '│', '┃')
            };
            for a in amin..aq1 {
                put(buf, a, mid, wl, s);
            }
            for a in aq3 + 1..=amax {
                put(buf, a, mid, wl, s);
            }
            let cw = (bw / 2).max(0);
            for c in mid - cw / 2..=mid + cw / 2 {
                put(buf, amin, c, cp, s);
                put(buf, amax, c, cp, s);
            }
            for a in aq1..=aq3 {
                for c in left..left + bw {
                    put(buf, a, c, '█', s);
                }
            }
            let ms = Style::default()
                .fg(self.theme.as_ref().map_or(Color::White, |t| t.foreground))
                .bg(color);
            for c in left..left + bw {
                put(buf, amed, c, ml, ms);
            }
            // notch: carve the box sides around the median CI
            if self.notched && st.n > 1 && bw >= 5 {
                let ci = 1.58 * (st.q3 - st.q1) / (st.n as f64).sqrt();
                let (a0, a1) = (to_a(st.median - ci).max(aq1), to_a(st.median + ci).min(aq3));
                for a in a0..=a1 {
                    put(buf, a, left, ' ', Style::default());
                    put(buf, a, left + bw - 1, ' ', Style::default());
                }
                if a0 > aq1 && a1 < aq3 {
                    for c in [left, left + bw - 1] {
                        put(buf, a0 - 1, c, if c == left { '┐' } else { '┌' }, s);
                        put(buf, a1 + 1, c, if c == left { '┘' } else { '└' }, s);
                    }
                }
            }
            if self.jitter && bw >= 3 {
                let ps =
                    Style::default().fg(self.theme.as_ref().map_or(Color::White, |t| t.foreground));
                for (k, &v) in st.points.iter().enumerate() {
                    if v < st.min || v > st.max {
                        continue;
                    }
                    // deterministic hash → column offset inside the box
                    let h = (k as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40;
                    let c = left + (h % bw as u64) as i32;
                    put(buf, to_a(v), c, '•', ps);
                }
            }
            if self.show_mean {
                if let Some(m) = st.mean.filter(|m| m.is_finite()) {
                    let ms = Style::default()
                        .fg(self.theme.as_ref().map_or(Color::White, |t| t.foreground))
                        .bg(color);
                    put(buf, to_a(m), mid, '◆', ms);
                }
            }
            if self.show_outliers {
                for &o in &st.outliers {
                    let os = match (&self.colors, &self.theme) {
                        (None, Some(t)) => Style::default().fg(t.bear),
                        _ => s,
                    };
                    put(buf, to_a(o), mid, '○', os);
                }
            }
        }
    }
}

widget_impls!(BoxPlot);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for BoxPlot {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(p: &BoxPlot, w: u16, h: u16) -> Buffer {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        p.render(area, &mut buf);
        buf
    }

    fn count(buf: &Buffer, sym: &str) -> usize {
        buf.content().iter().filter(|c| c.symbol() == sym).count()
    }

    #[test]
    fn quartiles_and_outlier() {
        let mut v: Vec<f64> = (1..=9).map(|x| x as f64).collect();
        v.push(100.0);
        let s = BoxStats::from_samples(&v, 1.5).unwrap();
        assert_eq!(s.median, 5.5);
        assert_eq!(s.q1, 3.25);
        assert_eq!(s.q3, 7.75);
        assert_eq!(s.max, 9.0);
        assert_eq!(s.outliers, vec![100.0]);
    }

    #[test]
    fn empty_and_nan() {
        assert!(BoxStats::from_samples(&[], 1.5).is_none());
        assert!(BoxStats::from_samples(&[f64::NAN], 1.5).is_none());
    }

    #[test]
    fn draws_box_median_outlier() {
        let p = BoxPlot::new().group("a", (1..=9).map(|x| x as f64).chain([100.0]));
        let b = render(&p, 30, 15);
        let p2 = BoxPlot::new().stats("a", BoxStats::new(0.0, 2.0, 5.0, 8.0, 10.0));
        assert!(count(&render(&p2, 30, 15), "█") > 0);
        assert!(count(&b, "━") > 0);
        assert_eq!(count(&b, "○"), 1);
    }

    #[test]
    fn mean_marker_toggle() {
        let p = BoxPlot::new().group("a", (1..=20).map(|x| x as f64));
        assert_eq!(count(&render(&p, 30, 15), "◆"), 1);
        assert_eq!(count(&render(&p.clone().show_mean(false), 30, 15), "◆"), 0);
        // precomputed stats without a mean draw no marker
        let q = BoxPlot::new().stats("a", BoxStats::new(0.0, 2.0, 5.0, 8.0, 10.0));
        assert_eq!(count(&render(&q, 30, 15), "◆"), 0);
        let q = BoxPlot::new().stats("a", BoxStats::new(0.0, 2.0, 5.0, 8.0, 10.0).mean(4.0));
        assert_eq!(count(&render(&q, 30, 15), "◆"), 1);
    }

    #[test]
    fn notches_change_box_only_with_sample_count() {
        let samples = (1..=40).map(|x| x as f64);
        let plain = render(&BoxPlot::new().group("a", samples.clone()), 30, 30);
        let notched = render(&BoxPlot::new().group("a", samples).notched(true), 30, 30);
        assert_ne!(plain, notched);
        let unknown = BoxPlot::new().stats("a", BoxStats::new(0.0, 2.0, 5.0, 8.0, 10.0));
        assert_eq!(
            render(&unknown, 30, 30),
            render(&unknown.clone().notched(true), 30, 30)
        );
    }

    #[test]
    fn jitter_draws_points_deterministically() {
        let p = BoxPlot::new()
            .group("a", (1..=60).map(|x| x as f64))
            .jitter(true);
        let a = render(&p, 30, 20);
        assert!(count(&a, "•") > 5);
        assert_eq!(a, render(&p, 30, 20));
        assert_eq!(
            count(
                &render(
                    &BoxPlot::new().group("a", (1..=60).map(|x| x as f64)),
                    30,
                    20
                ),
                "•"
            ),
            0
        );
    }

    #[test]
    fn horizontal_and_multi() {
        let p = BoxPlot::new()
            .stats("x", BoxStats::new(0.0, 2.0, 3.0, 4.0, 6.0))
            .stats("y", BoxStats::new(1.0, 3.0, 5.0, 7.0, 9.0))
            .orientation(BoxOrientation::Horizontal);
        let b = render(&p, 40, 12);
        assert!(count(&b, "┃") > 0);
    }

    #[cfg(feature = "theme")]
    #[test]
    fn theme_colors_and_override() {
        let t = Theme::nord();
        let p = BoxPlot::new().stats("a", BoxStats::new(0.0, 2.0, 5.0, 8.0, 10.0));
        let b = render(&p.clone().theme(&t), 30, 15);
        assert!(b
            .content()
            .iter()
            .any(|c| c.symbol() == "█" && c.fg == t.color(0)));
        assert_eq!(b[(0, 0)].bg, t.background.unwrap());
        let b = render(&p.theme(&t).colors(vec![Color::Red]), 30, 15);
        assert!(b
            .content()
            .iter()
            .any(|c| c.symbol() == "█" && c.fg == Color::Red));
        assert!(!b
            .content()
            .iter()
            .any(|c| c.symbol() == "█" && c.fg == t.color(0)));
    }

    #[test]
    fn tiny_areas_no_panic() {
        let p = BoxPlot::new()
            .group("a", [1.0, f64::NAN, 3.0])
            .stats("n", BoxStats::new(f64::NAN, 0.0, 0.0, 0.0, 0.0))
            .block(Block::bordered());
        for w in 0..8 {
            for h in 0..6 {
                render(&p, w, h);
            }
        }
        render(&BoxPlot::new(), 10, 5);
    }
}
