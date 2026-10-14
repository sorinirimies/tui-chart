//! Reusable chart annotations: reference lines, shaded regions and text labels.
//!
//! [`draw_annotations`] is chart-agnostic: callers supply closures mapping data
//! values to cell offsets relative to the plot rectangle.

use crate::layout::text_width;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

/// Geometry of an [`Annotation`] in data coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnnotationKind {
    /// Horizontal reference line at `y`.
    HLine(f64),
    /// Vertical reference line at `x`.
    VLine(f64),
    /// Vertical band shading `x0..=x1` over the full plot height.
    XRegion(f64, f64),
    /// Horizontal band shading `y0..=y1` over the full plot width.
    YRegion(f64, f64),
    /// Free text anchored at `(x, y)`.
    Label(f64, f64),
}

/// A reference line, shaded band or text label drawn over a plot.
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    /// Geometry in data coordinates.
    pub kind: AnnotationKind,
    /// Style (foreground for lines/text; regions use `bg`, falling back to `fg`).
    pub style: Style,
    /// Optional text label.
    pub text: Option<String>,
}

impl Annotation {
    fn make(kind: AnnotationKind, text: Option<String>) -> Self {
        Self {
            kind,
            style: Style::default().fg(Color::Yellow),
            text,
        }
    }

    /// Horizontal line at data `y`.
    pub fn hline(y: f64) -> Self {
        Self::make(AnnotationKind::HLine(y), None)
    }

    /// Vertical line at data `x`.
    pub fn vline(x: f64) -> Self {
        Self::make(AnnotationKind::VLine(x), None)
    }

    /// Shaded vertical band between `x0` and `x1`.
    pub fn x_region(x0: f64, x1: f64) -> Self {
        let mut a = Self::make(AnnotationKind::XRegion(x0, x1), None);
        a.style = Style::default().fg(Color::Gray).bg(Color::DarkGray);
        a
    }

    /// Shaded horizontal band between `y0` and `y1`.
    pub fn y_region(y0: f64, y1: f64) -> Self {
        let mut a = Self::make(AnnotationKind::YRegion(y0, y1), None);
        a.style = Style::default().fg(Color::Gray).bg(Color::DarkGray);
        a
    }

    /// Text label at `(x, y)`.
    pub fn label(x: f64, y: f64, text: impl Into<String>) -> Self {
        Self::make(AnnotationKind::Label(x, y), Some(text.into()))
    }

    /// Set the color (foreground; for regions also the band background).
    pub fn color(mut self, color: Color) -> Self {
        self.style = self.style.fg(color);
        if matches!(
            self.kind,
            AnnotationKind::XRegion(..) | AnnotationKind::YRegion(..)
        ) {
            self.style = self.style.bg(color);
        }
        self
    }

    /// Set the full style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Attach a text label.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
}

fn put_text(buf: &mut Buffer, plot: Rect, col: i32, row: i32, text: &str, style: Style) {
    if row < 0 || row >= plot.height as i32 {
        return;
    }
    let w = text_width(text) as i32;
    let start = col.clamp(0, (plot.width as i32 - w).max(0));
    let max = plot.width as i32 - start;
    if max <= 0 {
        return;
    }
    buf.set_stringn(
        plot.x + start as u16,
        plot.y + row as u16,
        text,
        max as usize,
        style,
    );
}

fn set_glyph(buf: &mut Buffer, x: u16, y: u16, ch: char, style: Style) {
    if let Some(c) = buf.cell_mut((x, y)) {
        c.set_char(ch).set_style(style);
    }
}

/// Draw `annotations` inside `plot`.
///
/// `x_cell` / `y_cell` map a data value to a cell column / row **relative to the
/// plot's top-left** (may fall outside the plot; return `None` if unmappable).
/// Call before drawing data so data glyphs stay on top. Never panics on
/// degenerate areas or non-finite values.
pub fn draw_annotations(
    buf: &mut Buffer,
    plot: Rect,
    annotations: &[Annotation],
    x_cell: &dyn Fn(f64) -> Option<i32>,
    y_cell: &dyn Fn(f64) -> Option<i32>,
) {
    if plot.width == 0 || plot.height == 0 {
        return;
    }
    let (w, h) = (plot.width as i32, plot.height as i32);
    for a in annotations {
        let text = a.text.as_deref();
        match a.kind {
            AnnotationKind::HLine(y) => {
                let Some(r) = y_cell(y).filter(|r| (0..h).contains(r)) else {
                    continue;
                };
                for c in 0..w {
                    set_glyph(buf, plot.x + c as u16, plot.y + r as u16, '╌', a.style);
                }
                if let Some(t) = text {
                    let row = if r > 0 { r - 1 } else { r };
                    put_text(buf, plot, w - text_width(t) as i32, row, t, a.style);
                }
            }
            AnnotationKind::VLine(x) => {
                let Some(c) = x_cell(x).filter(|c| (0..w).contains(c)) else {
                    continue;
                };
                for r in 0..h {
                    set_glyph(buf, plot.x + c as u16, plot.y + r as u16, '╎', a.style);
                }
                if let Some(t) = text {
                    let tw = text_width(t) as i32;
                    let col = if c + 1 + tw <= w { c + 1 } else { c - tw };
                    put_text(buf, plot, col, 0, t, a.style);
                }
            }
            AnnotationKind::XRegion(x0, x1) => {
                let (Some(c0), Some(c1)) = (x_cell(x0), x_cell(x1)) else {
                    continue;
                };
                let (lo, hi) = (c0.min(c1).max(0), c0.max(c1).min(w - 1));
                let bg = region_bg(a.style);
                for c in lo..=hi {
                    for r in 0..h {
                        shade(buf, plot.x + c as u16, plot.y + r as u16, bg);
                    }
                }
                if let (Some(t), true) = (text, lo <= hi) {
                    put_text(buf, plot, lo, 0, t, a.style);
                }
            }
            AnnotationKind::YRegion(y0, y1) => {
                let (Some(r0), Some(r1)) = (y_cell(y0), y_cell(y1)) else {
                    continue;
                };
                let (lo, hi) = (r0.min(r1).max(0), r0.max(r1).min(h - 1));
                let bg = region_bg(a.style);
                for r in lo..=hi {
                    for c in 0..w {
                        shade(buf, plot.x + c as u16, plot.y + r as u16, bg);
                    }
                }
                if let (Some(t), true) = (text, lo <= hi) {
                    put_text(buf, plot, 0, lo, t, a.style);
                }
            }
            AnnotationKind::Label(x, y) => {
                if let (Some(c), Some(r), Some(t)) = (x_cell(x), y_cell(y), text) {
                    if (0..w).contains(&c) {
                        put_text(buf, plot, c, r, t, a.style);
                    }
                }
            }
        }
    }
}

fn region_bg(style: Style) -> Color {
    style.bg.or(style.fg).unwrap_or(Color::DarkGray)
}

fn shade(buf: &mut Buffer, x: u16, y: u16, bg: Color) {
    if let Some(c) = buf.cell_mut((x, y)) {
        c.set_bg(bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(v: f64) -> Option<i32> {
        v.is_finite().then(|| v.round() as i32)
    }

    #[test]
    fn builders() {
        let a = Annotation::hline(1.0).color(Color::Red).text("lim");
        assert_eq!(a.kind, AnnotationKind::HLine(1.0));
        assert_eq!(a.style.fg, Some(Color::Red));
        assert_eq!(a.text.as_deref(), Some("lim"));
        let r = Annotation::x_region(1.0, 2.0).color(Color::Blue);
        assert_eq!(r.style.bg, Some(Color::Blue));
    }

    #[test]
    fn draws_lines_and_region() {
        let plot = Rect::new(1, 1, 10, 5);
        let mut buf = Buffer::empty(Rect::new(0, 0, 12, 7));
        let anns = [
            Annotation::hline(2.0),
            Annotation::vline(4.0).text("v"),
            Annotation::y_region(3.0, 4.0).color(Color::Blue),
        ];
        draw_annotations(&mut buf, plot, &anns, &map, &map);
        assert_eq!(buf[(2, 3)].symbol(), "╌");
        assert_eq!(buf[(5, 5)].symbol(), "╎");
        assert_eq!(buf[(6, 1)].symbol(), "v");
        assert_eq!(buf[(8, 4)].bg, Color::Blue);
    }

    #[test]
    fn out_of_range_and_nan_safe() {
        let plot = Rect::new(0, 0, 4, 3);
        let mut buf = Buffer::empty(plot);
        let anns = [
            Annotation::hline(99.0),
            Annotation::vline(f64::NAN),
            Annotation::x_region(-50.0, 50.0),
            Annotation::label(1.0, 1.0, "very long label text"),
            Annotation::label(-5.0, 1.0, "x"),
        ];
        draw_annotations(&mut buf, plot, &anns, &map, &map);
        draw_annotations(&mut buf, Rect::new(0, 0, 0, 0), &anns, &map, &map);
    }
}
