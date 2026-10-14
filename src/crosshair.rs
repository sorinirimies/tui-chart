//! Crosshair cursor with value tooltip, plus nearest-point lookup.

use crate::layout::text_width;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

/// A cursor position in data coordinates, drawn as guide lines + tooltip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crosshair {
    /// Cursor x in data coordinates.
    pub x: f64,
    /// Cursor y in data coordinates.
    pub y: f64,
    /// Style of the guide lines.
    pub style: Style,
    /// Style of the tooltip box.
    pub tooltip_style: Style,
    /// Draw the value tooltip (default true).
    pub tooltip: bool,
}

impl Crosshair {
    /// Crosshair at data `(x, y)`.
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            style: Style::default().fg(Color::Gray),
            tooltip_style: Style::default().fg(Color::White).bg(Color::DarkGray),
            tooltip: true,
        }
    }

    /// Copy with theme crosshair/tooltip colors for styles still at their
    /// defaults (explicitly set styles are kept).
    pub(crate) fn themed(&self, t: &crate::theme::Theme) -> Self {
        let d = Crosshair::new(self.x, self.y);
        let mut c = *self;
        if c.style == d.style {
            c.style = Style::default().fg(t.crosshair);
        }
        if c.tooltip_style == d.tooltip_style {
            c.tooltip_style = Style::default().fg(t.tooltip_fg).bg(t.tooltip_bg);
        }
        c
    }

    setter!(
        /// Guide line style.
        style: Style
    );

    setter!(
        /// Tooltip style.
        tooltip_style: Style
    );

    setter!(
        /// Show or hide the tooltip.
        tooltip: bool
    );
}

/// Result of a nearest-point lookup.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NearestPoint {
    /// Series index.
    pub series: usize,
    /// Point index within the series.
    pub index: usize,
    /// Point x (data coordinates).
    pub x: f64,
    /// Point y (data coordinates).
    pub y: f64,
}

/// Index and distance of the point closest to `target`, with each axis divided
/// by `scale` (so distances are comparable). Non-finite points are skipped.
pub fn nearest_point(
    points: impl IntoIterator<Item = (f64, f64)>,
    target: (f64, f64),
    scale: (f64, f64),
) -> Option<(usize, f64)> {
    let sx = if scale.0.is_finite() && scale.0 > 0.0 {
        scale.0
    } else {
        1.0
    };
    let sy = if scale.1.is_finite() && scale.1 > 0.0 {
        scale.1
    } else {
        1.0
    };
    points
        .into_iter()
        .enumerate()
        .filter(|(_, (x, y))| x.is_finite() && y.is_finite())
        .map(|(i, (x, y))| {
            let (dx, dy) = ((x - target.0) / sx, (y - target.1) / sy);
            (i, (dx * dx + dy * dy).sqrt())
        })
        .filter(|(_, d)| d.is_finite())
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

/// Draw guide lines through cell `(col, row)` (relative to `plot`) and an
/// optional tooltip with `lines`. Out-of-plot positions draw nothing.
pub(crate) fn draw(
    buf: &mut Buffer,
    plot: Rect,
    ch: &Crosshair,
    col: i32,
    row: i32,
    lines: &[String],
) {
    let (w, h) = (plot.width as i32, plot.height as i32);
    if w == 0 || h == 0 || !(0..w).contains(&col) || !(0..h).contains(&row) {
        return;
    }
    let mut put = |c: i32, r: i32, glyph: &str| {
        if let Some(cell) = buf.cell_mut((plot.x + c as u16, plot.y + r as u16)) {
            if matches!(cell.symbol(), " " | "·" | "╌" | "╎") {
                cell.set_symbol(glyph).set_style(ch.style);
            }
        }
    };
    for c in 0..w {
        put(c, row, "┄");
    }
    for r in 0..h {
        put(col, r, "┆");
    }
    if let Some(cell) = buf.cell_mut((plot.x + col as u16, plot.y + row as u16)) {
        if matches!(cell.symbol(), " " | "·" | "┄" | "┆") {
            cell.set_symbol("┼").set_style(ch.style);
        }
    }
    if !ch.tooltip || lines.is_empty() {
        return;
    }
    let tw = lines
        .iter()
        .map(|l| text_width(l) as i32)
        .max()
        .unwrap_or(0)
        + 2;
    let th = lines.len() as i32;
    if tw > w || th > h {
        return;
    }
    let mut x = col + 2;
    if x + tw > w {
        x = col - 1 - tw;
    }
    let mut y = row + 1;
    if y + th > h {
        y = row - th;
    }
    let (x, y) = (x.clamp(0, w - tw), y.clamp(0, h - th));
    for (i, l) in lines.iter().enumerate() {
        let (cx, cy) = (plot.x + x as u16, plot.y + (y + i as i32) as u16);
        for k in 0..tw {
            if let Some(cell) = buf.cell_mut((cx + k as u16, cy)) {
                cell.set_symbol(" ").set_style(ch.tooltip_style);
            }
        }
        buf.set_stringn(cx + 1, cy, l, (tw - 2) as usize, ch.tooltip_style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_basic() {
        let pts = [(0.0, 0.0), (5.0, 5.0), (f64::NAN, 1.0), (9.0, 9.0)];
        assert_eq!(nearest_point(pts, (4.0, 4.0), (1.0, 1.0)).unwrap().0, 1);
        assert_eq!(nearest_point(pts, (8.0, 8.0), (1.0, 1.0)).unwrap().0, 3);
        assert!(nearest_point([], (0.0, 0.0), (1.0, 1.0)).is_none());
        assert!(nearest_point([(f64::NAN, f64::NAN)], (0.0, 0.0), (0.0, 0.0)).is_none());
    }

    #[test]
    fn draws_guides_and_tooltip() {
        let plot = Rect::new(0, 0, 20, 8);
        let mut buf = Buffer::empty(plot);
        let ch = Crosshair::new(0.0, 0.0);
        draw(&mut buf, plot, &ch, 5, 3, &["x 1".into(), "y 2".into()]);
        assert_eq!(buf[(5, 3)].symbol(), "┼");
        assert_eq!(buf[(5, 0)].symbol(), "┆");
        assert_eq!(buf[(8, 4)].symbol(), "x");
    }

    #[test]
    fn edge_and_outside_safe() {
        let plot = Rect::new(0, 0, 6, 3);
        let mut buf = Buffer::empty(plot);
        let ch = Crosshair::new(0.0, 0.0);
        draw(&mut buf, plot, &ch, 5, 2, &["long tooltip line".into()]);
        draw(&mut buf, plot, &ch, -1, 9, &["a".into()]);
        draw(&mut buf, Rect::new(0, 0, 0, 0), &ch, 0, 0, &[]);
    }
}
