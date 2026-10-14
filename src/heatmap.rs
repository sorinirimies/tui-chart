//! Heatmap with selectable color gradients.

use crate::layout::{self, text_width};
use crate::theme::{self, ChartThemeExt, Theme};
use crate::util::format_compact;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// Color gradient presets.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Gradient {
    /// Perceptual purple → teal → yellow.
    #[default]
    Viridis,
    /// Black → red → yellow → white.
    Heat,
    /// Dark blue → cyan → white.
    Cool,
    /// Blue → white → red (diverging).
    RedBlue,
    /// Black → white.
    Grayscale,
    /// Custom RGB stops (evenly spaced, at least 2).
    Custom(Vec<(u8, u8, u8)>),
}

impl Gradient {
    fn stops(&self) -> Vec<(u8, u8, u8)> {
        match self {
            Gradient::Viridis => vec![
                (68, 1, 84),
                (59, 82, 139),
                (33, 145, 140),
                (94, 201, 98),
                (253, 231, 37),
            ],
            Gradient::Heat => vec![
                (0, 0, 0),
                (180, 0, 0),
                (255, 140, 0),
                (255, 255, 80),
                (255, 255, 255),
            ],
            Gradient::Cool => vec![
                (10, 20, 80),
                (30, 100, 200),
                (60, 200, 230),
                (255, 255, 255),
            ],
            Gradient::RedBlue => vec![(30, 60, 200), (255, 255, 255), (200, 30, 30)],
            Gradient::Grayscale => vec![(0, 0, 0), (255, 255, 255)],
            Gradient::Custom(s) if s.len() >= 2 => s.clone(),
            Gradient::Custom(_) => vec![(0, 0, 0), (255, 255, 255)],
        }
    }

    /// Color for `t` in `0..=1` (clamped; NaN → 0).
    pub fn color_at(&self, t: f64) -> Color {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        let stops = self.stops();
        let pos = t * (stops.len() - 1) as f64;
        let i = (pos.floor() as usize).min(stops.len() - 2);
        let f = pos - i as f64;
        let (a, b) = (stops[i], stops[i + 1]);
        let (r, g, bl) = crate::scale::mix_rgb(a, b, f);
        Color::Rgb(r, g, bl)
    }

    /// Next preset (for cycling in demos).
    pub fn next(&self) -> Gradient {
        match self {
            Gradient::Viridis => Gradient::Heat,
            Gradient::Heat => Gradient::Cool,
            Gradient::Cool => Gradient::RedBlue,
            Gradient::RedBlue => Gradient::Grayscale,
            _ => Gradient::Viridis,
        }
    }

    /// Preset name.
    pub fn name(&self) -> &'static str {
        match self {
            Gradient::Viridis => "viridis",
            Gradient::Heat => "heat",
            Gradient::Cool => "cool",
            Gradient::RedBlue => "red-blue",
            Gradient::Grayscale => "grayscale",
            Gradient::Custom(_) => "custom",
        }
    }
}

fn luminance(c: Color) -> f64 {
    match c {
        Color::Rgb(r, g, b) => 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64,
        _ => 128.0,
    }
}

/// Heatmap widget (rows × columns of values).
#[derive(Debug, Clone, Default)]
pub struct Heatmap {
    data: Vec<Vec<f64>>,
    row_labels: Vec<String>,
    col_labels: Vec<String>,
    gradient: Option<Gradient>,
    theme: Option<Theme>,
    style: Option<Style>,
    cell_width: u16,
    cell_height: u16,
    bounds: Option<(f64, f64)>,
    show_values: bool,
    legend: bool,
    block: Option<Block<'static>>,
}

impl Heatmap {
    /// Create from rows of values.
    pub fn new(data: Vec<Vec<f64>>) -> Self {
        Self {
            data,
            cell_width: 2,
            cell_height: 1,
            legend: true,
            ..Default::default()
        }
    }

    /// Row labels (left).
    pub fn row_labels<I, S>(mut self, l: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.row_labels = l.into_iter().map(Into::into).collect();
        self
    }

    /// Column labels (bottom).
    pub fn col_labels<I, S>(mut self, l: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.col_labels = l.into_iter().map(Into::into).collect();
        self
    }

    opt_setter!(
        /// Color gradient.
        gradient: Gradient
    );

    theme_setter!(
        /// Apply a [`Theme`]: gradient derived from its accent/palette (unless
        /// [`Heatmap::gradient`] was set), label colors and background fill.
        theme
    );

    base_style_setter!(style);

    fn grad(&self) -> Gradient {
        match (&self.gradient, &self.theme) {
            (Some(g), _) => g.clone(),
            (None, Some(t)) => t.gradient(),
            _ => Gradient::default(),
        }
    }

    fn label_color(&self) -> Color {
        self.theme.as_ref().map_or(Color::Gray, |t| t.muted)
    }

    fn dim_color(&self) -> Color {
        self.theme.as_ref().map_or(Color::DarkGray, |t| t.axis)
    }

    /// Cell width in columns (default 2).
    pub fn cell_width(mut self, w: u16) -> Self {
        self.cell_width = w.max(1);
        self
    }

    /// Cell height in rows (default 1).
    pub fn cell_height(mut self, h: u16) -> Self {
        self.cell_height = h.max(1);
        self
    }

    /// Fixed `(min, max)` color scale.
    pub fn bounds(mut self, min: f64, max: f64) -> Self {
        self.bounds = Some((min, max));
        self
    }

    setter!(
        /// Print values in cells when they fit.
        show_values: bool
    );

    setter!(
        /// Show the min–max gradient legend row.
        legend: bool
    );

    block_setter!(block, 'static);

    /// Resolved `(min, max)` of the color scale.
    pub fn scale(&self) -> (f64, f64) {
        if let Some(b) = self.bounds {
            return b;
        }
        let vals = self.data.iter().flatten().filter(|v| v.is_finite());
        let lo = vals.clone().cloned().fold(f64::INFINITY, f64::min);
        let hi = vals.cloned().fold(f64::NEG_INFINITY, f64::max);
        if lo.is_finite() {
            (lo, hi)
        } else {
            (0.0, 1.0)
        }
    }

    fn render_inner(&self, area: Rect, buf: &mut Buffer) {
        let mut inner = theme::begin(&self.block, &self.theme, self.style, area, buf);
        let grad = self.grad();
        let (lo, hi) = self.scale();
        let span = if hi > lo { hi - lo } else { 1.0 };
        if self.legend && inner.height > 4 {
            let y = inner.bottom() - 1;
            let (a, b) = (format_compact(lo), format_compact(hi));
            let (aw, bw) = (text_width(&a), text_width(&b));
            let lab = Style::default().fg(self.label_color());
            let bx = inner.right().saturating_sub(bw).max(inner.x);
            buf.set_stringn(inner.x, y, &a, inner.width as usize, lab);
            buf.set_stringn(bx, y, &b, (inner.right() - bx) as usize, lab);
            let bar_x = inner.x + aw + 1;
            let bar_end = inner.right().saturating_sub(bw + 1);
            if bar_end > bar_x {
                let n = (bar_end - bar_x) as f64;
                for (i, x) in (bar_x..bar_end).enumerate() {
                    let c = grad.color_at(i as f64 / (n - 1.0).max(1.0));
                    buf.set_string(x, y, " ", Style::default().bg(c));
                }
            }
            inner.height -= 1;
        }
        let yw = self
            .row_labels
            .iter()
            .map(|l| text_width(l))
            .max()
            .unwrap_or(0)
            .min(12);
        let Some(a) = layout::compute(inner, yw, !self.col_labels.is_empty(), false, false) else {
            return;
        };
        let plot = a.plot;
        let (cw, ch) = (self.cell_width, self.cell_height);
        let mut y_ticks = vec![];
        let mut x_ticks = vec![];
        for (r, row) in self.data.iter().enumerate() {
            let y = plot.y as u32 + r as u32 * ch as u32;
            if y + ch as u32 > plot.bottom() as u32 {
                break;
            }
            let y = y as u16;
            if let Some(l) = self.row_labels.get(r) {
                y_ticks.push((
                    y - plot.y + ch / 2,
                    l.chars().take(yw as usize).collect::<String>(),
                ));
            }
            for (c, &v) in row.iter().enumerate() {
                let x = plot.x as u32 + c as u32 * cw as u32;
                if x + cw as u32 > plot.right() as u32 {
                    break;
                }
                let x = x as u16;
                if r == 0 {
                    if let Some(l) = self.col_labels.get(c) {
                        x_ticks.push((x - plot.x + cw / 2, l.clone()));
                    }
                }
                let color = if v.is_finite() {
                    grad.color_at((v - lo) / span)
                } else {
                    self.dim_color()
                };
                let fg = if luminance(color) > 140.0 {
                    Color::Black
                } else {
                    Color::White
                };
                let text = if self.show_values && v.is_finite() {
                    let s = format_compact(v);
                    if text_width(&s) <= cw {
                        s
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                for dy in 0..ch {
                    for dx in 0..cw {
                        buf.set_string(x + dx, y + dy, " ", Style::default().bg(color));
                    }
                }
                if !text.is_empty() {
                    let tx = x + (cw - text_width(&text)) / 2;
                    buf.set_string(tx, y + ch / 2, &text, Style::default().bg(color).fg(fg));
                }
            }
        }
        layout::draw_axes(
            buf,
            &a,
            Style::default().fg(self.dim_color()),
            Style::default().fg(self.dim_color()),
        );
        let lab = Style::default().fg(self.label_color());
        layout::draw_y_labels(buf, a.y_labels, &y_ticks, lab);
        layout::draw_x_labels(buf, a.x_labels, &x_ticks, lab);
    }
}

impl Heatmap {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

widget_impls!(Heatmap);

impl tui_nuances::Themed for Heatmap {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn grid() -> Vec<Vec<f64>> {
        (0..4)
            .map(|r| (0..6).map(|c| (r * 6 + c) as f64).collect())
            .collect()
    }

    #[test]
    fn gradient_endpoints() {
        assert_eq!(Gradient::Grayscale.color_at(0.0), Color::Rgb(0, 0, 0));
        assert_eq!(Gradient::Grayscale.color_at(1.0), Color::Rgb(255, 255, 255));
        assert_eq!(Gradient::Grayscale.color_at(0.5), Color::Rgb(128, 128, 128));
        assert_eq!(Gradient::Grayscale.color_at(f64::NAN), Color::Rgb(0, 0, 0));
        assert_eq!(Gradient::Grayscale.color_at(9.0), Color::Rgb(255, 255, 255));
    }

    #[test]
    fn gradient_custom_and_cycle() {
        let g = Gradient::Custom(vec![(0, 0, 0), (100, 0, 0), (200, 0, 0)]);
        assert_eq!(g.color_at(0.5), Color::Rgb(100, 0, 0));
        assert_eq!(
            Gradient::Custom(vec![]).color_at(1.0),
            Color::Rgb(255, 255, 255)
        );
        let mut g = Gradient::Viridis;
        for _ in 0..5 {
            g = g.next();
        }
        assert_eq!(g, Gradient::Viridis);
        assert_eq!(Gradient::Heat.name(), "heat");
    }

    #[test]
    fn scale_auto_and_fixed() {
        assert_eq!(Heatmap::new(grid()).scale(), (0.0, 23.0));
        assert_eq!(Heatmap::new(grid()).bounds(-1.0, 1.0).scale(), (-1.0, 1.0));
        assert_eq!(Heatmap::new(vec![]).scale(), (0.0, 1.0));
    }

    #[test]
    fn renders_colored_cells_and_labels() {
        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);
        Heatmap::new(grid())
            .row_labels(["a", "b", "c", "d"])
            .col_labels(["1", "2", "3", "4", "5", "6"])
            .gradient(Gradient::Grayscale)
            .render(area, &mut buf);
        // first data cell is darkest, last is brightest
        let first = buf[(2, 0)].bg;
        let last = buf[(2 + 5 * 2, 3)].bg;
        assert_eq!(first, Color::Rgb(0, 0, 0));
        assert_eq!(last, Color::Rgb(255, 255, 255));
        assert_eq!(buf[(0, 0)].symbol(), "a");
    }

    #[test]
    fn legend_row_drawn() {
        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);
        Heatmap::new(grid()).render(area, &mut buf);
        assert_eq!(buf[(0, 9)].symbol(), "0");
        assert_eq!(buf[(39, 9)].symbol(), "3");
        let mut buf2 = Buffer::empty(area);
        Heatmap::new(grid()).legend(false).render(area, &mut buf2);
        assert_eq!(buf2[(0, 9)].symbol(), "└");
    }

    #[test]
    fn show_values_when_fit() {
        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);
        Heatmap::new(vec![vec![7.0, 123456.0]])
            .cell_width(3)
            .show_values(true)
            .render(area, &mut buf);
        let t: String = (0..40).map(|x| buf[(x, 0)].symbol().to_string()).collect();
        assert!(t.contains('7'));
        assert!(!t.contains("123"));
    }

    #[test]
    fn nan_cell_is_gray() {
        let area = Rect::new(0, 0, 20, 8);
        let mut buf = Buffer::empty(area);
        Heatmap::new(vec![vec![f64::NAN, 1.0]])
            .legend(false)
            .render(area, &mut buf);
        assert_eq!(buf[(1, 0)].bg, Color::DarkGray);
    }

    #[test]
    fn tiny_safe() {
        for (w, h) in [(0, 0), (2, 2), (6, 5)] {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            Heatmap::new(grid())
                .col_labels(["x"])
                .render(area, &mut buf);
        }
    }
}
