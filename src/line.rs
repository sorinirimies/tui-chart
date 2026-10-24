//! Line, step, area and scatter charts.

use crate::annotation::{draw_annotations, Annotation};
use crate::axis::Axis;
use crate::canvas::{Canvas, Marker};
use crate::crosshair::{self, nearest_point, Crosshair, NearestPoint};
use crate::heatmap::Gradient;
use crate::interact;
use crate::layout::{self, text_width};
use crate::theme::{self, Theme};
use crate::util::{format_compact, format_tick, nice_bounds};
use crate::PALETTE;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// How a series is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LineMode {
    /// Straight segments between points.
    #[default]
    Line,
    /// Horizontal-then-vertical steps (good for discrete state).
    Step,
    /// Line with a shaded fill down to the x axis.
    Area,
    /// Points only.
    Scatter,
}

/// Where the legend is drawn inside the plot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LegendPosition {
    /// No legend.
    None,
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    #[default]
    TopRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner.
    BottomRight,
}

/// One named data series of `(x, y)` points.
#[derive(Debug, Clone, Default)]
pub struct Series {
    pub(crate) name: String,
    pub(crate) data: Vec<(f64, f64)>,
    pub(crate) color: Option<Color>,
    pub(crate) mode: Option<LineMode>,
    pub(crate) y_err: Vec<f64>,
    pub(crate) band: Vec<(f64, f64)>,
    pub(crate) smooth: bool,
    pub(crate) secondary: bool,
    pub(crate) fill_gradient: Option<Gradient>,
}

impl Series {
    /// New empty series with a legend name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the `(x, y)` data.
    pub fn data(mut self, data: impl IntoIterator<Item = (f64, f64)>) -> Self {
        self.data = data.into_iter().collect();
        self
    }

    /// Plot values with x = 0, 1, 2, …
    pub fn values(mut self, values: impl IntoIterator<Item = f64>) -> Self {
        self.data = values
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as f64, v))
            .collect();
        self
    }

    opt_setter!(
        /// Series color (default: palette by index).
        color: Color
    );

    opt_setter!(
        /// Color an `Area` fill by value (bottom = low, top = high) instead of one flat color.
        fill_gradient: Gradient
    );

    opt_setter!(
        /// Override the chart's draw mode for this series.
        mode: LineMode
    );

    /// Symmetric error bars: `y ± err[i]` for point `i`.
    pub fn y_err(mut self, err: impl IntoIterator<Item = f64>) -> Self {
        self.y_err = err.into_iter().collect();
        self
    }

    /// Confidence band: `(lo, hi)` y-values for point `i`, shaded around the line.
    pub fn band(mut self, band: impl IntoIterator<Item = (f64, f64)>) -> Self {
        self.band = band.into_iter().collect();
        self
    }

    setter!(
        /// Smooth the line with Catmull-Rom spline interpolation (Line/Area modes).
        smooth: bool
    );

    setter!(
        /// Plot against the secondary (right) y axis.
        secondary_axis => secondary: bool
    );

    /// Series name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Number of points.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// True if the series has no points.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// All finite y extents (points, error bars, band edges).
    fn y_values(&self) -> impl Iterator<Item = f64> + '_ {
        self.data
            .iter()
            .enumerate()
            .filter(|(_, (x, y))| x.is_finite() && y.is_finite())
            .flat_map(|(i, &(_, y))| {
                let mut v = vec![y];
                if let Some(e) = self.y_err.get(i).filter(|e| e.is_finite()) {
                    v.push(y - e.abs());
                    v.push(y + e.abs());
                }
                if let Some(&(lo, hi)) = self.band.get(i) {
                    v.push(lo);
                    v.push(hi);
                }
                v
            })
            .filter(|v| v.is_finite())
    }
}

/// Linear or log10 value → 0..=1 fraction mapping.
#[derive(Debug, Clone, Copy)]
struct Scale {
    lo: f64,
    hi: f64,
    log: bool,
}

impl Scale {
    fn frac(&self, v: f64) -> Option<f64> {
        if !v.is_finite() {
            return None;
        }
        if self.log {
            if v <= 0.0 || self.lo <= 0.0 {
                return None;
            }
            let (l0, l1) = (self.lo.log10(), self.hi.log10());
            Some((v.log10() - l0) / (l1 - l0))
        } else {
            Some((v - self.lo) / (self.hi - self.lo))
        }
    }
}

/// Decade-aligned bounds from a raw positive range.
fn log_bounds(raw: Option<(f64, f64)>) -> (f64, f64) {
    match raw {
        None => (1.0, 10.0),
        Some((a, b)) => {
            let lo = 10f64.powf(a.log10().floor());
            let mut hi = 10f64.powf(b.log10().ceil());
            if hi <= lo {
                hi = lo * 10.0;
            }
            (lo, hi)
        }
    }
}

fn decade_label(k: i32) -> String {
    if (0..=5).contains(&k) {
        format!("{}", 10u64.pow(k as u32))
    } else if (-3..0).contains(&k) {
        let v = 10f64.powi(k);
        format_tick(v, v)
    } else {
        format!("1e{k}")
    }
}

/// Multi-series line / step / area / scatter chart with sub-cell resolution.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{LineChart, Series};
///
/// let chart = LineChart::new(vec![Series::new("squares").data((0..10).map(|i| (i as f64, (i * i) as f64)))]);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone, Default)]
pub struct LineChart {
    series: Vec<Series>,
    x_axis: Axis,
    y_axis: Axis,
    y2_axis: Axis,
    y_log: bool,
    smooth: bool,
    annotations: Vec<Annotation>,
    crosshair: Option<Crosshair>,
    block: Option<Block<'static>>,
    marker: Marker,
    mode: LineMode,
    legend: LegendPosition,
    legend_style: Option<Style>,
    theme: Option<Theme>,
    style: Option<Style>,
    hidden: Vec<usize>,
}

impl LineChart {
    /// Create a chart from series.
    pub fn new(series: Vec<Series>) -> Self {
        Self {
            series,
            ..Default::default()
        }
    }

    setter!(
        /// Replace the series.
        series: Vec<Series>
    );

    /// X axis configuration.
    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = axis;
        if let Some(t) = &self.theme {
            self.x_axis.theme_in_place(t);
        }
        self
    }

    /// Y axis configuration.
    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = axis;
        if let Some(t) = &self.theme {
            self.y_axis.theme_in_place(t);
        }
        self
    }

    /// Secondary (right) y axis configuration, used by `secondary_axis(true)` series.
    pub fn y2_axis(mut self, axis: Axis) -> Self {
        self.y2_axis = axis;
        if let Some(t) = &self.theme {
            self.y2_axis.theme_in_place(t);
        }
        self
    }

    setter!(
        /// Logarithmic (base 10) primary y axis with decade ticks. Non-positive
        /// values are not drawn.
        y_log: bool
    );

    setter!(
        /// Smooth all Line/Area series with Catmull-Rom splines.
        smooth: bool
    );

    /// Add an annotation (reference line, region or label).
    pub fn annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    setter!(
        /// Set (or clear) the crosshair cursor.
        crosshair: Option<Crosshair>
    );

    block_setter!(block, 'static);

    setter!(
        /// Glyph family (default braille).
        marker: Marker
    );

    setter!(
        /// Default draw mode for all series.
        mode: LineMode
    );

    /// Apply a [`Theme`]: axis/grid/border colors, background fill and the series
    /// palette. Explicit series colors and explicitly styled axes still win.
    #[cfg(feature = "theme")]
    pub fn theme(mut self, theme: impl Into<Theme>) -> Self {
        let t = theme.into();
        self.x_axis.theme_in_place(&t);
        self.y_axis.theme_in_place(&t);
        self.y2_axis.theme_in_place(&t);
        self.theme = Some(t);
        self
    }

    base_style_setter!(style);

    opt_setter!(
        /// Style of legend entry text (default: theme muted color / gray).
        legend_style: Style
    );

    fn series_color(&self, i: usize, s: &Series) -> Color {
        s.color.unwrap_or_else(|| match &self.theme {
            Some(t) => t.color(i),
            None => PALETTE[i % PALETTE.len()],
        })
    }

    setter!(
        /// Legend placement.
        legend: LegendPosition
    );

    /// Hide the series with these indices (axes rescale to the rest). Hidden
    /// series stay in the legend, dimmed, so they can be toggled back; see
    /// [`Interaction`](crate::Interaction).
    #[must_use]
    pub fn hidden(mut self, hidden: impl IntoIterator<Item = usize>) -> Self {
        self.hidden = hidden.into_iter().collect();
        self
    }

    fn is_hidden(&self, i: usize) -> bool {
        self.hidden.contains(&i)
    }

    fn has_secondary(&self) -> bool {
        self.series.iter().any(|s| s.secondary && !s.is_empty())
    }

    /// Raw `(min, max)` of y values of primary or secondary series.
    fn y_range(&self, secondary: bool, positive_only: bool) -> Option<(f64, f64)> {
        let mut r: Option<(f64, f64)> = None;
        for v in self
            .series
            .iter()
            .enumerate()
            .filter(|(i, s)| s.secondary == secondary && !self.is_hidden(*i))
            .map(|(_, s)| s)
            .flat_map(|s| s.y_values())
            .filter(|v| !positive_only || *v > 0.0)
        {
            r = Some(r.map_or((v, v), |(a, b)| (a.min(v), b.max(v))));
        }
        r
    }

    /// Resolved `(x_bounds, y_bounds)` (primary axis, data coordinates).
    pub fn bounds(&self) -> ((f64, f64), (f64, f64)) {
        let mut xs = (f64::INFINITY, f64::NEG_INFINITY);
        for &(x, _) in self
            .series
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.is_hidden(*i))
            .flat_map(|(_, s)| s.data.iter())
            .filter(|(x, y)| x.is_finite() && y.is_finite())
        {
            xs = (xs.0.min(x), xs.1.max(x));
        }
        let x = self.x_axis.bounds.unwrap_or_else(|| {
            if xs.0.is_finite() {
                if xs.1 <= xs.0 {
                    (xs.0 - 1.0, xs.1 + 1.0)
                } else {
                    xs
                }
            } else {
                (0.0, 1.0)
            }
        });
        let y = if self.y_log {
            match self.y_axis.bounds {
                Some((a, b)) if a > 0.0 && b > a => (a, b),
                _ => log_bounds(self.y_range(false, true)),
            }
        } else {
            self.y_axis.bounds.unwrap_or_else(|| {
                self.y_range(false, false)
                    .map_or((0.0, 1.0), |(a, b)| nice_bounds(a, b, self.y_axis.ticks))
            })
        };
        (x, y)
    }

    /// Bounds of the secondary y axis, or `None` when no series uses it.
    pub fn y2_bounds(&self) -> Option<(f64, f64)> {
        if !self.has_secondary() {
            return None;
        }
        Some(self.y2_axis.bounds.unwrap_or_else(|| {
            self.y_range(true, false)
                .map_or((0.0, 1.0), |(a, b)| nice_bounds(a, b, self.y2_axis.ticks))
        }))
    }

    fn scales(&self) -> (Scale, Scale, Option<Scale>) {
        let (xb, yb) = self.bounds();
        (
            Scale {
                lo: xb.0,
                hi: xb.1,
                log: false,
            },
            Scale {
                lo: yb.0,
                hi: yb.1,
                log: self.y_log,
            },
            self.y2_bounds().map(|b| Scale {
                lo: b.0,
                hi: b.1,
                log: false,
            }),
        )
    }

    /// Data point (over all series) closest to data position `(x, y)`, measured
    /// in plot-fraction space so both axes weigh equally. `y` is in primary-axis
    /// units; secondary series are compared by their position on the plot.
    pub fn nearest(&self, x: f64, y: f64) -> Option<NearestPoint> {
        let (sx, sy, sy2) = self.scales();
        let target = (sx.frac(x)?, sy.frac(y)?);
        let mut best: Option<(f64, NearestPoint)> = None;
        for (si, s) in self.series.iter().enumerate() {
            let ys = if s.secondary { sy2.unwrap_or(sy) } else { sy };
            let pts = s
                .data
                .iter()
                .map(|&(px, py)| match (sx.frac(px), ys.frac(py)) {
                    (Some(a), Some(b)) => (a, b),
                    _ => (f64::NAN, f64::NAN),
                });
            if let Some((i, d)) = nearest_point(pts, target, (1.0, 1.0)) {
                if best.is_none_or(|(bd, _)| d < bd) {
                    let (px, py) = s.data[i];
                    best = Some((
                        d,
                        NearestPoint {
                            series: si,
                            index: i,
                            x: px,
                            y: py,
                        },
                    ));
                }
            }
        }
        best.map(|b| b.1)
    }

    fn y_marks(&self, b: (f64, f64)) -> Vec<(f64, String)> {
        if !self.y_log || self.y_axis.labels.is_some() {
            return self.y_axis.tick_marks(b);
        }
        let (l0, l1) = (b.0.log10(), b.1.log10());
        if l1.is_nan() || l1 <= l0 {
            return vec![];
        }
        let (k0, k1) = ((l0 - 1e-9).ceil() as i32, (l1 + 1e-9).floor() as i32);
        let n = (i64::from(k1) - i64::from(k0) + 1).max(0) as usize;
        let stride = n.div_ceil(self.y_axis.ticks.max(2)).max(1);
        (k0..=k1)
            .step_by(stride)
            .map(|k| {
                let label = match self.y_axis.format(10f64.powi(k)) {
                    Some(l) => l,
                    None => decade_label(k),
                };
                (f64::from(k) - l0, label)
            })
            .map(|(d, l)| (d / (l1 - l0), l))
            .collect()
    }

    fn render_inner(&self, area: Rect, buf: &mut Buffer) {
        let inner = theme::begin(&self.block, &self.theme, self.style, area, buf);
        let (xb, yb) = self.bounds();
        if xb.1 <= xb.0 || yb.1 <= yb.0 {
            return;
        }
        let (sx, sy, sy2) = self.scales();
        let y_marks = self.y_marks(yb);
        let x_marks = self.x_axis.tick_marks(xb);
        let y2_marks = match (&sy2, self.y2_bounds()) {
            (Some(_), Some(b)) if b.1 > b.0 => self.y2_axis.tick_marks(b),
            _ => vec![],
        };
        let y2_reserve = if sy2.is_some() {
            y2_marks
                .iter()
                .map(|(_, l)| text_width(l))
                .max()
                .unwrap_or(0)
                + 1
        } else {
            0
        };
        let inner = Rect {
            width: inner.width.saturating_sub(y2_reserve),
            ..inner
        };
        let yw = y_marks
            .iter()
            .map(|(_, l)| text_width(l))
            .max()
            .unwrap_or(0);
        let Some(a) = layout::compute(
            inner,
            yw,
            !x_marks.is_empty(),
            self.x_axis.title.is_some(),
            self.y_axis.title.is_some(),
        ) else {
            return;
        };
        let (rx, ry) = self.marker.resolution();
        let plot = a.plot;
        let y_ticks: Vec<(u16, String)> = y_marks
            .iter()
            .map(|(f, l)| (layout::frac_row(*f, plot.height, ry), l.clone()))
            .collect();
        let x_ticks: Vec<(u16, String)> = x_marks
            .iter()
            .map(|(f, l)| (layout::frac_col(*f, plot.width, rx), l.clone()))
            .collect();

        let grid_rows: Vec<u16> = if self.y_axis.grid {
            y_ticks.iter().map(|t| t.0).collect()
        } else {
            vec![]
        };
        let grid_cols: Vec<u16> = if self.x_axis.grid {
            x_ticks.iter().map(|t| t.0).collect()
        } else {
            vec![]
        };
        layout::draw_grid(
            buf,
            plot,
            &grid_cols,
            &grid_rows,
            self.x_axis.grid_style,
            self.x_axis.grid_kind,
        );

        // cell mappings relative to the plot (unclamped, may be outside)
        let clampi = |v: f64| v.round().clamp(-20_000.0, 20_000.0) as i32;
        let (nw, nh) = (
            (plot.width as f64 * rx as f64 - 1.0).max(0.0),
            (plot.height as f64 * ry as f64 - 1.0).max(0.0),
        );
        let x_cell = |x: f64| sx.frac(x).map(|f| clampi(f * nw).div_euclid(rx as i32));
        let y_cell = |y: f64| {
            sy.frac(y)
                .map(|f| clampi((1.0 - f) * nh).div_euclid(ry as i32))
        };
        draw_annotations(buf, plot, &self.annotations, &x_cell, &y_cell);

        let mut canvas = Canvas::new(plot.width, plot.height, self.marker);
        let (pw, ph) = (canvas.px_w(), canvas.px_h());
        let proj = |x: f64, y: f64, sec: bool| -> Option<(i32, i32)> {
            let ys = if sec { sy2.as_ref()? } else { &sy };
            let (fx, fy) = (sx.frac(x)?, ys.frac(y)?);
            Some((
                clampi(fx * (pw - 1) as f64),
                clampi((1.0 - fy) * (ph - 1) as f64),
            ))
        };
        for (i, s) in self.series.iter().enumerate() {
            if self.is_hidden(i) {
                continue;
            }
            let color = self.series_color(i, s);
            let mode = s.mode.unwrap_or(self.mode);
            let sec = s.secondary && sy2.is_some();
            let mut pts: Vec<(i32, i32)> = Vec::with_capacity(s.data.len());
            let mut band: Vec<(i32, i32, i32)> = Vec::new();
            for (k, &(x, y)) in s.data.iter().enumerate() {
                if !x.is_finite() || !y.is_finite() {
                    continue;
                }
                let Some(p) = proj(x, y, sec) else { continue };
                pts.push(p);
                if interact::recording() {
                    let cx = plot.x
                        + p.0
                            .div_euclid(rx as i32)
                            .clamp(0, i32::from(plot.width) - 1) as u16;
                    let cy = plot.y
                        + p.1
                            .div_euclid(ry as i32)
                            .clamp(0, i32::from(plot.height) - 1) as u16;
                    interact::record_point(Rect::new(cx, cy, 1, 1), i, k, || {
                        (
                            s.name.clone(),
                            vec![
                                format!("x {}", format_compact(x)),
                                format!("y {}", format_compact(y)),
                            ],
                        )
                    });
                }
                if let Some(&(lo, hi)) = s.band.get(k) {
                    let lo_p = proj(x, lo, sec).map_or(ph - 1, |q| q.1);
                    let hi_p = proj(x, hi, sec).map_or(ph - 1, |q| q.1);
                    band.push((p.0, lo_p, hi_p));
                }
                if let Some(e) = s.y_err.get(k).filter(|e| e.is_finite()) {
                    let e = e.abs();
                    let lo_p = proj(x, y - e, sec).map_or(ph - 1, |q| q.1);
                    let hi_p = proj(x, y + e, sec).map_or(ph - 1, |q| q.1);
                    canvas.line(p.0, lo_p, p.0, hi_p, color);
                    canvas.line(p.0 - 2, lo_p, p.0 + 2, lo_p, color);
                    canvas.line(p.0 - 2, hi_p, p.0 + 2, hi_p, color);
                }
            }
            fill_band(&mut canvas, &band, color);
            if (s.smooth || self.smooth) && matches!(mode, LineMode::Line | LineMode::Area) {
                pts = catmull_rom(&pts);
            }
            if mode != LineMode::Scatter && pts.len() > canvas.px_w() as usize * 4 {
                pts = decimate(&pts);
            }
            draw_series(&mut canvas, &pts, mode, color, s.fill_gradient.as_ref());
        }
        canvas.render(buf, plot);

        if let Some(ch) = &self.crosshair {
            if let (Some(c), Some(r)) = (x_cell(ch.x), y_cell(ch.y)) {
                let themed = self.theme.as_ref().map(|t| ch.with_theme(t));
                let ch = themed.as_ref().unwrap_or(ch);
                crosshair::draw(buf, plot, ch, c, r, &self.tooltip_lines(ch));
            }
        }

        layout::draw_axes(buf, &a, self.x_axis.style, self.y_axis.style);
        layout::draw_y_labels(buf, a.y_labels, &y_ticks, self.y_axis.label_style);
        layout::draw_x_labels(buf, a.x_labels, &x_ticks, self.x_axis.label_style);
        if y2_reserve > 0 {
            for y in plot.y..a.axis_row {
                buf.set_string(plot.right(), y, "│", self.y2_axis.style);
            }
            buf.set_string(plot.right(), a.axis_row, "┘", self.x_axis.style);
            let mut last: Option<u16> = None;
            for (f, label) in &y2_marks {
                let row = layout::frac_row(*f, plot.height, ry);
                if last == Some(row) {
                    continue;
                }
                last = Some(row);
                buf.set_stringn(
                    plot.right() + 1,
                    plot.y + row,
                    label,
                    (y2_reserve - 1) as usize,
                    self.y2_axis.label_style,
                );
            }
        }
        layout::draw_titles(
            buf,
            inner,
            &a,
            self.x_axis
                .title
                .as_deref()
                .map(|t| (t, self.x_axis.label_style)),
            self.y_axis
                .title
                .as_deref()
                .map(|t| (t, self.y_axis.label_style)),
        );
        self.draw_legend(buf, plot);
    }

    fn tooltip_lines(&self, ch: &Crosshair) -> Vec<String> {
        let mut lines = vec![
            format!("x {}", format_compact(ch.x)),
            format!("y {}", format_compact(ch.y)),
        ];
        for s in self.series.iter().filter(|s| !s.name.is_empty()) {
            if let Some((i, _)) =
                nearest_point(s.data.iter().copied(), (ch.x, 0.0), (1.0, f64::INFINITY))
            {
                lines.push(format!("{} {}", s.name, format_compact(s.data[i].1)));
            }
        }
        lines
    }

    fn draw_legend(&self, buf: &mut Buffer, plot: Rect) {
        if self.legend == LegendPosition::None {
            return;
        }
        let entries: Vec<(&str, Color)> = self
            .series
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.name.is_empty())
            .map(|(i, s)| (s.name.as_str(), self.series_color(i, s)))
            .collect();
        let ids: Vec<usize> = self
            .series
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.name.is_empty())
            .map(|(i, _)| i)
            .collect();
        let w = entries
            .iter()
            .map(|(n, _)| text_width(n) + 2)
            .max()
            .unwrap_or(0);
        if w == 0 || w > plot.width {
            return;
        }
        let x = match self.legend {
            LegendPosition::TopLeft | LegendPosition::BottomLeft => plot.x,
            _ => plot.right() - w,
        };
        let label = self.legend_style.unwrap_or_else(|| {
            Style::default().fg(self.theme.as_ref().map_or(Color::Gray, |t| t.muted))
        });
        for (i, (name, color)) in entries.iter().enumerate() {
            let row = i as u16;
            if row >= plot.height {
                break;
            }
            let y = match self.legend {
                LegendPosition::BottomLeft | LegendPosition::BottomRight => {
                    plot.bottom() - 1 - (entries.len() as u16 - 1 - row).min(plot.height - 1)
                }
                _ => plot.y + row,
            };
            let off = self.is_hidden(ids[i]);
            let (dot, text) = if off {
                (
                    "○",
                    label.add_modifier(
                        ratatui::style::Modifier::CROSSED_OUT | ratatui::style::Modifier::DIM,
                    ),
                )
            } else {
                ("●", label)
            };
            buf.set_string(x, y, dot, Style::default().fg(*color));
            buf.set_string(x + 2, y, name, text);
            interact::record_legend(Rect::new(x, y, w, 1), ids[i], name);
        }
    }
}

/// Catmull-Rom spline through `pts` (pixel space), densified for line drawing.
fn catmull_rom(pts: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let n = pts.len();
    let mut out = Vec::with_capacity(n * 4);
    for i in 0..n - 1 {
        let p0 = pts[i.saturating_sub(1)];
        let (p1, p2) = (pts[i], pts[i + 1]);
        let p3 = pts[(i + 2).min(n - 1)];
        out.push(p1);
        let span = (p2.0 - p1.0).abs().max((p2.1 - p1.1).abs());
        let steps = (span / 2).clamp(1, 64);
        for k in 1..steps {
            let t = k as f64 / steps as f64;
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: i32, b: i32, c: i32, d: i32| {
                let (a, b, c, d) = (a as f64, b as f64, c as f64, d as f64);
                (0.5 * (2.0 * b
                    + (c - a) * t
                    + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
                    + (3.0 * b - a - 3.0 * c + d) * t3))
                    .round()
                    .clamp(-20_000.0, 20_000.0) as i32
            };
            out.push((f(p0.0, p1.0, p2.0, p3.0), f(p0.1, p1.1, p2.1, p3.1)));
        }
    }
    out.push(pts[n - 1]);
    out
}

/// Dotted shading between `(px, lo_py, hi_py)` samples.
fn fill_band(canvas: &mut Canvas, band: &[(i32, i32, i32)], color: Color) {
    let (pw, ph) = (canvas.px_w(), canvas.px_h());
    let column = |px: i32, a: i32, b: i32, canvas: &mut Canvas| {
        if !(0..pw).contains(&px) {
            return;
        }
        for py in a.min(b).max(0)..=a.max(b).min(ph - 1) {
            if (px + py) % 2 == 0 {
                canvas.set(px, py, color);
            }
        }
    };
    if band.len() == 1 {
        column(band[0].0, band[0].1, band[0].2, canvas);
    }
    for w in band.windows(2) {
        let (l, r) = if w[0].0 <= w[1].0 {
            (w[0], w[1])
        } else {
            (w[1], w[0])
        };
        for px in l.0.max(0)..=r.0.min(pw - 1) {
            let t = if r.0 == l.0 {
                0.0
            } else {
                (px - l.0) as f64 / (r.0 - l.0) as f64
            };
            let lerp = |a: i32, b: i32| (a as f64 + t * (b - a) as f64).round() as i32;
            column(px, lerp(l.1, r.1), lerp(l.2, r.2), canvas);
        }
    }
}

/// Collapse x-sorted points to at most 4 per pixel column (first, min, max,
/// last, in original order), which draws the same pixels as the full series
/// but with far fewer segments/fills. Unsorted input is returned unchanged.
fn decimate(pts: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if pts.windows(2).any(|w| w[1].0 < w[0].0) {
        return pts.to_vec();
    }
    let mut out = Vec::with_capacity(pts.len() / 8 + 8);
    let mut i = 0;
    while i < pts.len() {
        let x = pts[i].0;
        let mut j = i;
        let (mut lo, mut hi) = (i, i);
        while j < pts.len() && pts[j].0 == x {
            if pts[j].1 < pts[lo].1 {
                lo = j;
            }
            if pts[j].1 > pts[hi].1 {
                hi = j;
            }
            j += 1;
        }
        let mut keep = [i, lo, hi, j - 1];
        keep.sort_unstable();
        let mut last = usize::MAX;
        for k in keep {
            if k != last {
                out.push(pts[k]);
                last = k;
            }
        }
        i = j;
    }
    out
}

fn draw_series(
    canvas: &mut Canvas,
    pts: &[(i32, i32)],
    mode: LineMode,
    color: Color,
    grad: Option<&Gradient>,
) {
    let bottom = canvas.px_h() - 1;
    match mode {
        LineMode::Scatter => {
            for &(x, y) in pts {
                canvas.blob(x, y, color);
            }
        }
        LineMode::Line | LineMode::Area | LineMode::Step => {
            if mode == LineMode::Area {
                for w in pts.windows(2) {
                    fill_segment(canvas, w[0], w[1], bottom, color, grad);
                }
            }
            if pts.len() == 1 {
                canvas.set(pts[0].0, pts[0].1, color);
            }
            for w in pts.windows(2) {
                let ((x0, y0), (x1, y1)) = (w[0], w[1]);
                if mode == LineMode::Step {
                    canvas.line(x0, y0, x1, y0, color);
                    canvas.line(x1, y0, x1, y1, color);
                } else {
                    canvas.line(x0, y0, x1, y1, color);
                }
            }
        }
    }
}

fn fill_segment(
    canvas: &mut Canvas,
    a: (i32, i32),
    b: (i32, i32),
    bottom: i32,
    color: Color,
    grad: Option<&Gradient>,
) {
    let (lo, hi) = if a.0 <= b.0 { (a, b) } else { (b, a) };
    let start = lo.0.max(0);
    let end = hi.0.min(canvas.px_w() - 1);
    // stipple (every other dot) only where a cell has >1 dot per axis
    let (rx, ry) = canvas.marker().resolution();
    let (sx, sy) = (if rx > 1 { 2 } else { 1 }, if ry > 1 { 2 } else { 1 });
    for px in start..=end {
        let t = if hi.0 == lo.0 {
            0.0
        } else {
            (px - lo.0) as f64 / (hi.0 - lo.0) as f64
        };
        let py = (lo.1 as f64 + t * (hi.1 - lo.1) as f64).round() as i32;
        for yy in py.max(0)..=bottom {
            if px % sx == 0 && yy % sy == 0 {
                let c = match grad {
                    Some(g) if bottom > 0 => g.color_at((bottom - yy) as f64 / bottom as f64),
                    _ => color,
                };
                canvas.set(px, yy, c);
            }
        }
    }
}

impl LineChart {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

widget_impls!(LineChart);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for LineChart {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn chart() -> LineChart {
        LineChart::new(vec![
            Series::new("a").values([1.0, 3.0, 2.0, 5.0]),
            Series::new("b").data([(0.0, 0.0), (3.0, 4.0)]),
        ])
    }

    #[test]
    fn bounds_auto() {
        let ((x0, x1), (y0, y1)) = chart().bounds();
        assert_eq!((x0, x1), (0.0, 3.0));
        assert!(y0 <= 0.0 && y1 >= 5.0);
    }

    #[test]
    fn bounds_fixed_and_empty() {
        let c = LineChart::default().x_axis(Axis::new().bounds(2.0, 4.0));
        assert_eq!(c.bounds().0, (2.0, 4.0));
        assert_eq!(c.bounds().1, (0.0, 1.0));
    }

    #[test]
    fn bounds_single_point_padded() {
        let c = LineChart::new(vec![Series::new("p").data([(1.0, 1.0)])]);
        let ((x0, x1), (y0, y1)) = c.bounds();
        assert!(x1 > x0 && y1 > y0);
    }

    #[test]
    fn nan_points_ignored() {
        let c = LineChart::new(vec![Series::new("p").data([
            (f64::NAN, 1.0),
            (1.0, 2.0),
            (2.0, 4.0),
        ])]);
        assert_eq!(c.bounds().0, (1.0, 2.0));
    }

    #[test]
    fn series_helpers() {
        let s = Series::new("x")
            .values([1.0, 2.0])
            .color(Color::Red)
            .mode(LineMode::Step);
        assert_eq!(s.name(), "x");
        assert_eq!(s.len(), 2);
        assert!(!s.is_empty());
        assert!(Series::new("e").is_empty());
    }

    #[test]
    fn renders_without_panic_all_modes() {
        for mode in [
            LineMode::Line,
            LineMode::Step,
            LineMode::Area,
            LineMode::Scatter,
        ] {
            for marker in [
                Marker::Braille,
                Marker::HalfBlock,
                Marker::Block,
                Marker::Dot,
            ] {
                let area = Rect::new(0, 0, 50, 14);
                let mut buf = Buffer::empty(area);
                chart()
                    .mode(mode)
                    .marker(marker)
                    .x_axis(Axis::new().title("x").grid(true))
                    .y_axis(Axis::new().title("y").grid(true))
                    .block(Block::bordered().title("t"))
                    .render(area, &mut buf);
            }
        }
    }

    #[test]
    fn tiny_area_is_safe() {
        for (w, h) in [(0, 0), (1, 1), (5, 3), (9, 4)] {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            chart().render(area, &mut buf);
        }
    }

    #[test]
    fn draws_data_and_legend() {
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        chart().render(area, &mut buf);
        let text: String = (0..12)
            .flat_map(|y| (0..40).map(move |x| (x, y)))
            .map(|p| buf[p].symbol().to_string())
            .collect();
        assert!(text.chars().any(|c| ('\u{2801}'..='\u{28ff}').contains(&c)));
        assert!(text.contains('●'));
        assert!(text.contains('└'));
    }

    #[test]
    fn legend_none_hides() {
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        chart().legend(LegendPosition::None).render(area, &mut buf);
        let text: String = (0..12)
            .flat_map(|y| (0..40).map(move |x| (x, y)))
            .map(|p| buf[p].symbol().to_string())
            .collect();
        assert!(!text.contains('●'));
    }

    fn text_of(buf: &Buffer, w: u16, h: u16) -> String {
        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .map(|p| buf[p].symbol().to_string())
            .collect()
    }

    #[test]
    fn log_bounds_decades_and_labels() {
        let c = LineChart::new(vec![Series::new("a").data([
            (0.0, 3.0),
            (1.0, 4500.0),
            (2.0, -1.0),
        ])])
        .y_log(true);
        assert_eq!(c.bounds().1, (1.0, 10_000.0));
        let m = c.y_marks((1.0, 10_000.0));
        assert_eq!(m.first().unwrap().1, "1");
        assert_eq!(m.last().unwrap().1, "10000");
        assert!((m[2].0 - 0.5).abs() < 1e-9);
        assert_eq!(LineChart::default().y_log(true).bounds().1, (1.0, 10.0));
    }

    #[test]
    fn secondary_axis_bounds_and_render() {
        let c = LineChart::new(vec![
            Series::new("a").values([1.0, 2.0]),
            Series::new("b").values([100.0, 900.0]).secondary_axis(true),
        ]);
        assert!(c.bounds().1 .1 < 10.0);
        let b2 = c.y2_bounds().unwrap();
        assert!(b2.1 >= 900.0);
        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);
        c.render(area, &mut buf);
        assert!(text_of(&buf, 40, 10).contains('┘'));
        assert!(LineChart::default().y2_bounds().is_none());
    }

    #[test]
    fn err_band_extend_bounds() {
        let c = LineChart::new(vec![Series::new("a")
            .values([1.0, 2.0])
            .y_err([10.0, 10.0])]);
        assert!(c.bounds().1 .1 >= 12.0);
        let c = LineChart::new(vec![Series::new("a")
            .values([1.0, 2.0])
            .band([(-5.0, 1.0), (0.0, 3.0)])]);
        assert!(c.bounds().1 .0 <= -5.0);
    }

    #[test]
    fn catmull_densifies() {
        let p = catmull_rom(&[(0, 0), (20, 10), (40, 0), (60, 10)]);
        assert!(p.len() > 4);
        assert_eq!(p[0], (0, 0));
        assert_eq!(*p.last().unwrap(), (60, 10));
        assert_eq!(catmull_rom(&[(1, 1)]), vec![(1, 1)]);
    }

    #[test]
    fn nearest_finds_point() {
        let n = chart().nearest(3.0, 4.0).unwrap();
        assert_eq!((n.series, n.index), (1, 1));
        assert!(LineChart::default().nearest(0.0, 0.0).is_none());
    }

    #[test]
    fn features_render_safely_everywhere() {
        let c = LineChart::new(vec![
            Series::new("a")
                .values([1.0, 0.0, f64::NAN, 5.0])
                .y_err([0.5, f64::NAN, 1.0])
                .smooth(true),
            Series::new("b")
                .values([10.0, 20.0, 5.0])
                .band([(1.0, 30.0), (2.0, 40.0), (0.0, 9.0)])
                .secondary_axis(true),
        ])
        .y_log(true)
        .annotation(Annotation::hline(2.0).text("lim"))
        .annotation(Annotation::x_region(0.5, 1.5))
        .crosshair(Some(Crosshair::new(1.0, 2.0)));
        for (w, h) in [(0, 0), (1, 1), (5, 3), (12, 6), (60, 20)] {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            (&c).render(area, &mut buf);
        }
    }

    #[test]
    fn crosshair_draws_tooltip() {
        let area = Rect::new(0, 0, 50, 16);
        let mut buf = Buffer::empty(area);
        chart()
            .crosshair(Some(Crosshair::new(1.0, 2.0)))
            .render(area, &mut buf);
        let t = text_of(&buf, 50, 16);
        assert!(t.contains('┼'));
        assert!(t.contains("x 1"));
    }

    #[cfg(feature = "theme")]
    #[test]
    fn crosshair_uses_theme_colors() {
        let th = Theme::dracula();
        let area = Rect::new(0, 0, 50, 16);
        let mut buf = Buffer::empty(area);
        chart()
            .theme(&th)
            .crosshair(Some(Crosshair::new(1.0, 2.0)))
            .render(area, &mut buf);
        assert!(buf
            .content()
            .iter()
            .any(|c| c.symbol() == "┼" && c.fg == th.crosshair));
        let mut buf = Buffer::empty(area);
        let own = Style::default().fg(Color::Red);
        chart()
            .theme(&th)
            .crosshair(Some(Crosshair::new(1.0, 2.0).style(own)))
            .render(area, &mut buf);
        assert!(buf
            .content()
            .iter()
            .any(|c| c.symbol() == "┼" && c.fg == Color::Red));
    }

    #[test]
    fn decimate_keeps_extremes_and_order() {
        let pts = vec![(0, 5), (0, 1), (0, 9), (0, 4), (1, 2), (2, 3)];
        let d = decimate(&pts);
        assert_eq!(d, vec![(0, 5), (0, 1), (0, 9), (0, 4), (1, 2), (2, 3)]);
        let unsorted = vec![(3, 1), (1, 1)];
        assert_eq!(decimate(&unsorted), unsorted);
    }

    #[test]
    fn interaction_hits_hidden_series_and_legend() {
        use crate::{HitKind, HitMap};
        let two = || {
            LineChart::new(vec![
                Series::new("a").data([(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)]),
                Series::new("b").data([(0.0, 3.0), (1.0, 1.0), (2.0, 2.0)]),
            ])
        };
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        let hits = HitMap::capture(two(), area, &mut buf);
        let pts = hits
            .hits()
            .iter()
            .filter(|h| h.kind == HitKind::Point)
            .count();
        assert_eq!(pts, 6);
        assert_eq!(
            hits.hits()
                .iter()
                .filter(|h| h.kind == HitKind::Legend)
                .count(),
            2
        );
        let p = &hits.hits()[0];
        assert_eq!(p.title, "a");
        assert!(p.lines[0].starts_with("x "));
        // hiding series 0 drops its points but keeps its legend entry
        let mut buf = Buffer::empty(area);
        let hits = HitMap::capture(two().hidden([0]), area, &mut buf);
        assert!(hits
            .hits()
            .iter()
            .all(|h| h.kind == HitKind::Legend || h.series == 1));
        assert!(hits
            .hits()
            .iter()
            .any(|h| h.kind == HitKind::Legend && h.series == 0));
        // all hidden still renders
        let mut buf = Buffer::empty(area);
        two().hidden([0, 1]).render(area, &mut buf);
    }
}
