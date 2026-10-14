//! Line, step, area and scatter charts.

use crate::axis::Axis;
use crate::canvas::{Canvas, Marker};
use crate::layout::{self, text_width};
use crate::util::nice_bounds;
use crate::PALETTE;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

/// How a series is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

    /// Series color (default: palette by index).
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Override the chart's draw mode for this series.
    pub fn mode(mut self, mode: LineMode) -> Self {
        self.mode = Some(mode);
        self
    }

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
}

/// Multi-series line / step / area / scatter chart with sub-cell resolution.
#[derive(Debug, Clone, Default)]
pub struct LineChart {
    series: Vec<Series>,
    x_axis: Axis,
    y_axis: Axis,
    block: Option<Block<'static>>,
    marker: Marker,
    mode: LineMode,
    legend: LegendPosition,
}

impl LineChart {
    /// Create a chart from series.
    pub fn new(series: Vec<Series>) -> Self {
        Self {
            series,
            ..Default::default()
        }
    }

    /// Replace the series.
    pub fn series(mut self, series: Vec<Series>) -> Self {
        self.series = series;
        self
    }

    /// X axis configuration.
    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = axis;
        self
    }

    /// Y axis configuration.
    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = axis;
        self
    }

    /// Wrap the chart in a block.
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Glyph family (default braille).
    pub fn marker(mut self, marker: Marker) -> Self {
        self.marker = marker;
        self
    }

    /// Default draw mode for all series.
    pub fn mode(mut self, mode: LineMode) -> Self {
        self.mode = mode;
        self
    }

    /// Legend placement.
    pub fn legend(mut self, pos: LegendPosition) -> Self {
        self.legend = pos;
        self
    }

    /// Resolved `(x_bounds, y_bounds)`.
    pub fn bounds(&self) -> ((f64, f64), (f64, f64)) {
        let mut xs = (f64::INFINITY, f64::NEG_INFINITY);
        let mut ys = (f64::INFINITY, f64::NEG_INFINITY);
        for &(x, y) in self
            .series
            .iter()
            .flat_map(|s| s.data.iter())
            .filter(|(x, y)| x.is_finite() && y.is_finite())
        {
            xs = (xs.0.min(x), xs.1.max(x));
            ys = (ys.0.min(y), ys.1.max(y));
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
        let y = self.y_axis.bounds.unwrap_or_else(|| {
            if ys.0.is_finite() {
                nice_bounds(ys.0, ys.1, self.y_axis.ticks)
            } else {
                (0.0, 1.0)
            }
        });
        (x, y)
    }

    fn render_inner(&self, area: Rect, buf: &mut Buffer) {
        let inner = layout::render_block(&self.block, area, buf);
        let (xb, yb) = self.bounds();
        if xb.1 <= xb.0 || yb.1 <= yb.0 {
            return;
        }
        let y_marks = self.y_axis.tick_marks(yb);
        let x_marks = self.x_axis.tick_marks(xb);
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
        layout::draw_grid(buf, plot, &grid_cols, &grid_rows, self.x_axis.grid_style);

        let mut canvas = Canvas::new(plot.width, plot.height, self.marker);
        let (pw, ph) = (canvas.px_w(), canvas.px_h());
        let project = |x: f64, y: f64| -> (i32, i32) {
            let px = (x - xb.0) / (xb.1 - xb.0) * (pw - 1) as f64;
            let py = (1.0 - (y - yb.0) / (yb.1 - yb.0)) * (ph - 1) as f64;
            (
                px.round().clamp(-20_000.0, 20_000.0) as i32,
                py.round().clamp(-20_000.0, 20_000.0) as i32,
            )
        };
        for (i, s) in self.series.iter().enumerate() {
            let color = s.color.unwrap_or(PALETTE[i % PALETTE.len()]);
            let mode = s.mode.unwrap_or(self.mode);
            let pts: Vec<(i32, i32)> = s
                .data
                .iter()
                .filter(|(x, y)| x.is_finite() && y.is_finite())
                .map(|&(x, y)| project(x, y))
                .collect();
            draw_series(&mut canvas, &pts, mode, color);
        }
        canvas.render(buf, plot);

        layout::draw_axes(buf, &a, self.x_axis.style, self.y_axis.style);
        layout::draw_y_labels(buf, a.y_labels, &y_ticks, self.y_axis.label_style);
        layout::draw_x_labels(buf, a.x_labels, &x_ticks, self.x_axis.label_style);
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

    fn draw_legend(&self, buf: &mut Buffer, plot: Rect) {
        if self.legend == LegendPosition::None {
            return;
        }
        let entries: Vec<(&str, Color)> = self
            .series
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.name.is_empty())
            .map(|(i, s)| {
                (
                    s.name.as_str(),
                    s.color.unwrap_or(PALETTE[i % PALETTE.len()]),
                )
            })
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
            buf.set_string(x, y, "●", Style::default().fg(*color));
            buf.set_string(x + 2, y, name, Style::default().fg(Color::Gray));
        }
    }
}

fn draw_series(canvas: &mut Canvas, pts: &[(i32, i32)], mode: LineMode, color: Color) {
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
                    fill_segment(canvas, w[0], w[1], bottom, color);
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

fn fill_segment(canvas: &mut Canvas, a: (i32, i32), b: (i32, i32), bottom: i32, color: Color) {
    let (lo, hi) = if a.0 <= b.0 { (a, b) } else { (b, a) };
    let start = lo.0.max(0);
    let end = hi.0.min(canvas.px_w() - 1);
    for px in start..=end {
        let t = if hi.0 == lo.0 {
            0.0
        } else {
            (px - lo.0) as f64 / (hi.0 - lo.0) as f64
        };
        let py = (lo.1 as f64 + t * (hi.1 - lo.1) as f64).round() as i32;
        for yy in py.max(0)..=bottom {
            if px % 2 == 0 && yy % 2 == 0 {
                canvas.set(px, yy, color);
            }
        }
    }
}

impl Widget for &LineChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

impl Widget for LineChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
