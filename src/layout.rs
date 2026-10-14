//! Shared plot-area layout and axis/label drawing.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Widget},
};
use unicode_width::UnicodeWidthStr;

/// Rectangles produced by [`compute`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct Areas {
    pub plot: Rect,
    pub y_labels: Rect,
    pub x_labels: Rect,
    pub axis_col: u16,
    pub axis_row: u16,
    pub y_title_row: Option<u16>,
    pub x_title_row: Option<u16>,
}

/// Render an optional block and return the inner area.
pub(crate) fn render_block(block: &Option<Block<'static>>, area: Rect, buf: &mut Buffer) -> Rect {
    match block {
        Some(b) => {
            let inner = b.inner(area);
            b.clone().render(area, buf);
            inner
        }
        None => area,
    }
}

/// Split `area` into plot area, label gutters and axis lines.
pub(crate) fn compute(
    area: Rect,
    y_label_w: u16,
    x_labels: bool,
    x_title: bool,
    y_title: bool,
) -> Option<Areas> {
    let fixed_h = 1 + x_labels as u16 + x_title as u16 + y_title as u16;
    if area.height < fixed_h + 2 || area.width < y_label_w + 1 + 2 {
        return None;
    }
    let mut top = area.y;
    let y_title_row = if y_title {
        top += 1;
        Some(area.y)
    } else {
        None
    };
    let mut bottom = area.bottom();
    let x_title_row = if x_title {
        bottom -= 1;
        Some(bottom)
    } else {
        None
    };
    let label_row = if x_labels {
        bottom -= 1;
        bottom
    } else {
        bottom
    };
    bottom -= 1;
    let axis_row = bottom;
    let plot_x = area.x + y_label_w + 1;
    let plot = Rect::new(plot_x, top, area.right() - plot_x, axis_row - top);
    Some(Areas {
        plot,
        y_labels: Rect::new(area.x, top, y_label_w, plot.height),
        x_labels: Rect::new(plot.x, label_row, plot.width, x_labels as u16),
        axis_col: plot_x - 1,
        axis_row,
        y_title_row,
        x_title_row,
    })
}

/// Terminal row (relative to the plot) for a 0..=1 fraction at `res` dots per cell.
pub(crate) fn frac_row(frac: f64, h: u16, res: u16) -> u16 {
    let n = (h as f64 * res as f64).max(1.0);
    (((1.0 - frac) * (n - 1.0)).round().clamp(0.0, n - 1.0) as u16) / res
}

/// Terminal column (relative to the plot) for a 0..=1 fraction at `res` dots per cell.
pub(crate) fn frac_col(frac: f64, w: u16, res: u16) -> u16 {
    let n = (w as f64 * res as f64).max(1.0);
    ((frac * (n - 1.0)).round().clamp(0.0, n - 1.0) as u16) / res
}

pub(crate) fn text_width(s: &str) -> u16 {
    UnicodeWidthStr::width(s).min(u16::MAX as usize) as u16
}

/// Draw the L-shaped axis lines.
pub(crate) fn draw_axes(buf: &mut Buffer, a: &Areas, x_style: Style, y_style: Style) {
    for y in a.plot.y..a.axis_row {
        buf.set_string(a.axis_col, y, "│", y_style);
    }
    buf.set_string(a.axis_col, a.axis_row, "└", x_style);
    for x in a.plot.x..a.plot.right() {
        buf.set_string(x, a.axis_row, "─", x_style);
    }
}

/// Draw y-axis labels right-aligned in the gutter. `ticks` = (row relative to gutter, text).
pub(crate) fn draw_y_labels(buf: &mut Buffer, gutter: Rect, ticks: &[(u16, String)], style: Style) {
    let mut last: Option<u16> = None;
    for (row, text) in ticks {
        if *row >= gutter.height || last == Some(*row) {
            continue;
        }
        last = Some(*row);
        let w = text_width(text).min(gutter.width);
        buf.set_string(gutter.right() - w, gutter.y + row, text, style);
    }
}

/// Draw x-axis labels centred on their columns, skipping overlaps.
pub(crate) fn draw_x_labels(buf: &mut Buffer, row: Rect, ticks: &[(u16, String)], style: Style) {
    if row.height == 0 {
        return;
    }
    let mut next_free = row.x;
    for (col, text) in ticks {
        let w = text_width(text);
        if w == 0 || w > row.width {
            continue;
        }
        let centre = row.x + col.min(&row.width.saturating_sub(1));
        let mut start = centre.saturating_sub(w / 2).max(row.x);
        if start + w > row.right() {
            start = row.right() - w;
        }
        if start < next_free {
            continue;
        }
        buf.set_string(start, row.y, text, style);
        next_free = start + w + 1;
    }
}

/// Dotted grid lines at the given plot-relative columns/rows (only on empty cells).
pub(crate) fn draw_grid(buf: &mut Buffer, plot: Rect, cols: &[u16], rows: &[u16], style: Style) {
    for r in rows {
        if *r >= plot.height {
            continue;
        }
        for x in plot.x..plot.right() {
            put_grid(buf, x, plot.y + r, style);
        }
    }
    for c in cols {
        if *c >= plot.width {
            continue;
        }
        for y in plot.y..plot.bottom() {
            put_grid(buf, plot.x + c, y, style);
        }
    }
}

fn put_grid(buf: &mut Buffer, x: u16, y: u16, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        if cell.symbol() == " " {
            cell.set_char('·').set_style(style);
        }
    }
}

/// Draw axis titles.
pub(crate) fn draw_titles(
    buf: &mut Buffer,
    area: Rect,
    a: &Areas,
    x_title: Option<(&str, Style)>,
    y_title: Option<(&str, Style)>,
) {
    if let (Some(row), Some((t, s))) = (a.y_title_row, y_title) {
        buf.set_stringn(area.x, row, t, area.width as usize, s);
    }
    if let (Some(row), Some((t, s))) = (a.x_title_row, x_title) {
        let w = text_width(t).min(area.width);
        let x = area.x + (area.width - w) / 2;
        buf.set_stringn(x, row, t, w as usize, s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_layout() {
        let a = compute(Rect::new(0, 0, 40, 12), 4, true, true, true).unwrap();
        assert_eq!(a.y_title_row, Some(0));
        assert_eq!(a.x_title_row, Some(11));
        assert_eq!(a.x_labels.y, 10);
        assert_eq!(a.axis_row, 9);
        assert_eq!(a.plot, Rect::new(5, 1, 35, 8));
        assert_eq!(a.axis_col, 4);
    }

    #[test]
    fn compute_too_small() {
        assert!(compute(Rect::new(0, 0, 4, 12), 4, true, false, false).is_none());
        assert!(compute(Rect::new(0, 0, 40, 3), 4, true, false, false).is_none());
    }

    #[test]
    fn fractions() {
        assert_eq!(frac_row(1.0, 10, 4), 0);
        assert_eq!(frac_row(0.0, 10, 4), 9);
        assert_eq!(frac_col(0.0, 10, 2), 0);
        assert_eq!(frac_col(1.0, 10, 2), 9);
    }

    #[test]
    fn x_labels_skip_overlap() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        let row = Rect::new(0, 0, 20, 1);
        let ticks = vec![
            (2, "abcd".to_string()),
            (4, "efgh".to_string()),
            (15, "z".to_string()),
        ];
        draw_x_labels(&mut buf, row, &ticks, Style::default());
        let s: String = (0..20).map(|x| buf[(x, 0)].symbol().to_string()).collect();
        assert!(s.contains("abcd"));
        assert!(!s.contains("efgh"));
        assert!(s.contains('z'));
    }

    #[test]
    fn y_labels_right_aligned() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 3));
        draw_y_labels(
            &mut buf,
            Rect::new(0, 0, 4, 3),
            &[(1, "42".to_string())],
            Style::default(),
        );
        assert_eq!(buf[(2, 1)].symbol(), "4");
        assert_eq!(buf[(3, 1)].symbol(), "2");
    }

    #[test]
    fn grid_only_on_empty() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 4, 3));
        buf.set_string(1, 1, "x", Style::default());
        draw_grid(&mut buf, Rect::new(0, 0, 4, 3), &[], &[1], Style::default());
        assert_eq!(buf[(0, 1)].symbol(), "·");
        assert_eq!(buf[(1, 1)].symbol(), "x");
    }
}
