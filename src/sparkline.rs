//! Inline sparkline using block characters `▁▂▃▄▅▆▇█`.

use crate::canvas::{Canvas, Marker};
use crate::theme::{self, Theme};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

const GLYPHS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Rendering style of a [`Sparkline`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SparklineStyle {
    /// Filled columns made of block characters.
    #[default]
    Bar,
    /// Braille line connecting the points.
    Line,
}

/// Compact trend chart, one column per data point (newest points kept when too wide).
#[derive(Debug, Clone)]
pub struct Sparkline<'a> {
    data: Vec<f64>,
    min: Option<f64>,
    max: Option<f64>,
    baseline: Option<f64>,
    style: SparklineStyle,
    color: Option<Color>,
    theme: Option<Theme>,
    theme_gradient: bool,
    base: Option<Style>,
    gradient: Vec<Color>,
    last_color: Option<Color>,
    min_color: Option<Color>,
    max_color: Option<Color>,
    marker: Marker,
    block: Option<Block<'a>>,
}

impl<'a> Sparkline<'a> {
    /// Create a sparkline from values. Non-finite values are skipped.
    pub fn new(data: impl IntoIterator<Item = f64>) -> Self {
        Self {
            data: data.into_iter().collect(),
            min: None,
            max: None,
            baseline: None,
            style: SparklineStyle::Bar,
            color: None,
            theme: None,
            theme_gradient: false,
            base: None,
            gradient: Vec::new(),
            last_color: None,
            min_color: None,
            max_color: None,
            marker: Marker::Braille,
            block: None,
        }
    }
    opt_setter!(
        /// Fix the lower bound of the scale (default: data minimum).
        min: f64
    );
    opt_setter!(
        /// Fix the upper bound of the scale (default: data maximum).
        max: f64
    );
    opt_setter!(
        /// Value bars grow from this baseline (bar style only).
        baseline: f64
    );
    setter!(
        /// Bar or line style.
        style: SparklineStyle
    );
    opt_setter!(
        /// Base color.
        color: Color
    );
    theme_setter!(
        /// Apply a [`Theme`]: base color = accent, background fill. An explicit
        /// [`Sparkline::color`] wins.
        theme
    );
    setter!(
        /// Color by value along the theme gradient (needs a theme; an explicit
        /// [`Sparkline::gradient`] wins).
        theme_gradient: bool
    );
    base_style_setter!(base);
    /// Color gradient by value, from low to high (overrides base color).
    pub fn gradient(mut self, colors: impl IntoIterator<Item = Color>) -> Self {
        self.gradient = colors.into_iter().collect();
        self
    }
    opt_setter!(
        /// Highlight the last point.
        highlight_last => last_color: Color
    );
    opt_setter!(
        /// Highlight the minimum point.
        highlight_min => min_color: Color
    );
    opt_setter!(
        /// Highlight the maximum point.
        highlight_max => max_color: Color
    );
    setter!(
        /// Glyph family used for dot drawing (default [`Marker::Braille`]).
        marker: Marker
    );

    block_setter!(block, 'a);

    fn draw(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let start = self.data.len().saturating_sub(area.width as usize);
        let pts: Vec<(usize, f64)> = self.data[start..]
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, v)| v.is_finite())
            .collect();
        if pts.is_empty() {
            return;
        }
        let dmin = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let dmax = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let lo = self.min.filter(|v| v.is_finite()).unwrap_or(dmin);
        let mut hi = self.max.filter(|v| v.is_finite()).unwrap_or(dmax);
        if hi <= lo {
            hi = lo + 1.0;
        }
        let norm = |v: f64| ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
        let n_last = pts.last().map(|p| p.0);
        let base = self
            .color
            .unwrap_or_else(|| self.theme.as_ref().map_or(Color::Cyan, |t| t.accent));
        let themed: Vec<Color> = match (&self.theme, self.theme_gradient) {
            (Some(t), true) if self.gradient.is_empty() => {
                let g = t.gradient();
                (0..5).map(|i| g.color_at(i as f64 / 4.0)).collect()
            }
            _ => Vec::new(),
        };
        let grad = if self.gradient.is_empty() {
            &themed
        } else {
            &self.gradient
        };
        let color_of = |i: usize, v: f64| {
            let mut c = base;
            if !grad.is_empty() {
                let k = (norm(v) * (grad.len() - 1) as f64).round() as usize;
                c = grad[k.min(grad.len() - 1)];
            }
            if v == dmax {
                c = self.max_color.unwrap_or(c);
            }
            if v == dmin {
                c = self.min_color.unwrap_or(c);
            }
            if Some(i) == n_last {
                c = self.last_color.unwrap_or(c);
            }
            c
        };
        match self.style {
            SparklineStyle::Bar => {
                let levels = area.height as f64 * 8.0;
                let base = self.baseline.map(norm).map(|b| (b * levels).round() as i64);
                for &(i, v) in &pts {
                    let x = area.x + i as u16;
                    let top = ((norm(v) * levels).round() as i64).max(1);
                    let (a, b) = match base {
                        Some(b) => (b.min(top), b.max(top)),
                        None => (0, top),
                    };
                    let col = color_of(i, v);
                    for row in 0..area.height as i64 {
                        let (r0, r1) = (row * 8, row * 8 + 8);
                        let cov_lo = a.max(r0) - r0;
                        let cov_hi = b.min(r1) - r0;
                        if cov_hi <= cov_lo {
                            continue;
                        }
                        let y = area.y + area.height - 1 - row as u16;
                        let Some(cell) = buf.cell_mut((x, y)) else {
                            continue;
                        };
                        let ch = if cov_lo == 0 && cov_hi < 8 {
                            GLYPHS[(cov_hi - 1) as usize]
                        } else if cov_lo > 0 && cov_hi == 8 {
                            // upper part only: approximate with full block
                            '█'
                        } else {
                            '█'
                        };
                        cell.set_char(ch).set_fg(col);
                    }
                }
            }
            SparklineStyle::Line => {
                let mut cv = Canvas::new(area.width, area.height, self.marker);
                let pw = cv.px_w();
                let ph = cv.px_h();
                let pos = |i: usize, v: f64| {
                    (
                        (i as i32 * self.marker.resolution().0 as i32).min(pw - 1),
                        ph - 1 - (norm(v) * (ph - 1) as f64).round() as i32,
                    )
                };
                let mut prev: Option<(i32, i32)> = None;
                for &(i, v) in &pts {
                    let p = pos(i, v);
                    let col = color_of(i, v);
                    match prev {
                        Some(q) => cv.line(q.0, q.1, p.0, p.1, col),
                        None => cv.set(p.0, p.1, col),
                    }
                    prev = Some(p);
                }
                cv.render(buf, area);
            }
        }
    }
}

impl Sparkline<'_> {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        self.draw(inner, buf);
    }
}

widget_impls!(Sparkline<'_>);

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn sym(buf: &Buffer, x: u16, y: u16) -> String {
        buf[(x, y)].symbol().to_string()
    }

    #[test]
    fn single_row_levels() {
        let area = Rect::new(0, 0, 3, 1);
        let mut buf = Buffer::empty(area);
        Sparkline::new([0.0, 4.0, 8.0]).render(area, &mut buf);
        assert_eq!(sym(&buf, 0, 0), "▁");
        assert_eq!(sym(&buf, 2, 0), "█");
    }

    #[test]
    fn highlights_and_gradient() {
        let area = Rect::new(0, 0, 3, 1);
        let mut buf = Buffer::empty(area);
        Sparkline::new([1.0, 2.0, 3.0])
            .highlight_last(Color::Red)
            .highlight_min(Color::Blue)
            .render(area, &mut buf);
        assert_eq!(buf[(2, 0)].fg, Color::Red);
        assert_eq!(buf[(0, 0)].fg, Color::Blue);
    }

    #[test]
    fn tall_bars_fill_rows() {
        let area = Rect::new(0, 0, 2, 3);
        let mut buf = Buffer::empty(area);
        Sparkline::new([0.0, 10.0]).min(0.0).render(area, &mut buf);
        assert_eq!(sym(&buf, 1, 0), "█");
        assert_eq!(sym(&buf, 1, 2), "█");
    }

    #[test]
    fn line_style_and_degenerate() {
        let area = Rect::new(0, 0, 5, 2);
        let mut buf = Buffer::empty(area);
        Sparkline::new([1.0, 2.0, 1.0])
            .style(SparklineStyle::Line)
            .render(area, &mut buf);
        Sparkline::new([f64::NAN, 1.0, f64::INFINITY]).render(area, &mut buf);
        Sparkline::new(Vec::new()).render(area, &mut buf);
        Sparkline::new([1.0]).render(Rect::new(0, 0, 0, 0), &mut buf);
        Sparkline::new([5.0, 5.0]).render(area, &mut buf);
    }

    #[test]
    fn keeps_newest_when_narrow() {
        let area = Rect::new(0, 0, 2, 1);
        let mut buf = Buffer::empty(area);
        Sparkline::new([8.0, 0.0, 8.0])
            .min(0.0)
            .max(8.0)
            .render(area, &mut buf);
        assert_eq!(sym(&buf, 1, 0), "█");
        assert_eq!(sym(&buf, 0, 0), "▁");
    }
}
