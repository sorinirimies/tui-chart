//! Slope chart: compare two states per item with a connecting line.

use crate::canvas::{Canvas, Marker};
use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::{dim_text, format_compact, pick};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Block,
};

/// One line of a [`SlopeChart`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlopeItem {
    /// Item label.
    pub label: String,
    /// Value on the left axis.
    pub start: f64,
    /// Value on the right axis.
    pub end: f64,
    /// Explicit color.
    pub color: Option<Color>,
}

impl SlopeItem {
    /// Create an item.
    pub fn new(label: impl Into<String>, start: f64, end: f64) -> Self {
        Self {
            label: label.into(),
            start,
            end,
            color: None,
        }
    }

    /// Explicit color (beats trend coloring).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

/// Slope chart widget.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{SlopeChart, SlopeItem};
///
/// let chart = SlopeChart::new([SlopeItem::new("a", 1.0, 3.0), SlopeItem::new("b", 4.0, 2.0)]).titles("2023", "2024");
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct SlopeChart {
    items: Vec<SlopeItem>,
    titles: (String, String),
    trend_colors: bool,
    marker: Marker,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl SlopeChart {
    /// Create a chart.
    pub fn new(items: impl IntoIterator<Item = SlopeItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            titles: (String::new(), String::new()),
            trend_colors: true,
            marker: Marker::default(),
            theme: None,
            base: None,
            block: None,
        }
    }

    /// Column titles shown above the two sides (e.g. years).
    pub fn titles(mut self, left: impl Into<String>, right: impl Into<String>) -> Self {
        self.titles = (left.into(), right.into());
        self
    }

    setter!(
        /// Color lines green/red by trend instead of by palette (default on).
        trend_colors: bool
    );
    setter!(
        /// Dot resolution of the lines.
        marker: Marker
    );
    theme_setter!(
        /// Apply a [`Theme`]; explicit item colors win.
        theme
    );
    base_style_setter!(base);
    block_setter!(block, 'static);

    fn color_of(&self, i: usize, it: &SlopeItem) -> Color {
        it.color.unwrap_or_else(|| {
            if self.trend_colors {
                match it.end.total_cmp(&it.start) {
                    std::cmp::Ordering::Greater => Color::Green,
                    std::cmp::Ordering::Less => Color::Red,
                    std::cmp::Ordering::Equal => Color::Gray,
                }
            } else {
                pick(&self.theme, i)
            }
        })
    }

    /// Spread rows so labels never share a line.
    fn spread(mut rows: Vec<(usize, i32)>, top: i32, bottom: i32) -> Vec<(usize, i32)> {
        rows.sort_by_key(|r| r.1);
        let mut last = top - 1;
        for r in rows.iter_mut() {
            r.1 = r.1.max(last + 1);
            last = r.1;
        }
        let mut next = bottom + 1;
        for r in rows.iter_mut().rev() {
            r.1 = r.1.min(next - 1);
            next = r.1;
        }
        rows
    }

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 16 || inner.height < 3 {
            return;
        }
        let items: Vec<(usize, &SlopeItem)> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.start.is_finite() && i.end.is_finite())
            .collect();
        if items.is_empty() {
            return;
        }
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for (_, i) in &items {
            lo = lo.min(i.start.min(i.end));
            hi = hi.max(i.start.max(i.end));
        }
        let span = (hi - lo).max(f64::EPSILON);
        let (dim, text) = dim_text(&self.theme);

        let has_titles = !(self.titles.0.is_empty() && self.titles.1.is_empty());
        let top = inner.y + u16::from(has_titles);
        let rows_h = inner.bottom() - top;
        if rows_h < 1 {
            return;
        }
        let side = |left: bool| -> usize {
            items
                .iter()
                .map(|(_, i)| {
                    i.label.chars().count()
                        + 1
                        + format_compact(if left { i.start } else { i.end })
                            .chars()
                            .count()
                })
                .max()
                .unwrap_or(0)
        };
        let side_w = (side(true).max(side(false)) as u16).min(inner.width / 3);
        let cv_x = inner.x + side_w + 1;
        let cv_w = inner.width.saturating_sub(2 * side_w + 2);
        if cv_w < 4 {
            return;
        }
        let row_of =
            |v: f64| top as i32 + ((1.0 - (v - lo) / span) * (rows_h - 1) as f64).round() as i32;

        let mut cv = Canvas::new(cv_w, rows_h, self.marker);
        let (pw, ph) = (cv.px_w(), cv.px_h());
        let (_, ry) = self.marker.resolution();
        let ypx = |v: f64| {
            ((1.0 - (v - lo) / span) * (ph - ry as i32) as f64).round() as i32 + ry as i32 / 2
        };
        for (i, it) in &items {
            let c = self.color_of(*i, it);
            cv.line(0, ypx(it.start), pw - 1, ypx(it.end), c);
        }
        cv.render(buf, Rect::new(cv_x, top, cv_w, rows_h));

        let l_rows = Self::spread(
            items
                .iter()
                .enumerate()
                .map(|(k, (_, i))| (k, row_of(i.start)))
                .collect(),
            top as i32,
            inner.bottom() as i32 - 1,
        );
        let r_rows = Self::spread(
            items
                .iter()
                .enumerate()
                .map(|(k, (_, i))| (k, row_of(i.end)))
                .collect(),
            top as i32,
            inner.bottom() as i32 - 1,
        );
        let visible = |y: &i32| *y >= top as i32 && *y < inner.bottom() as i32;
        for (k, y) in l_rows.into_iter().filter(|(_, y)| visible(y)) {
            let (i, it) = items[k];
            let c = self.color_of(i, it);
            let s = format!("{} {}", it.label, format_compact(it.start));
            let s: String = s.chars().take(side_w as usize).collect();
            let x = inner.x + side_w - s.chars().count() as u16;

            crate::interact::record_point(
                Rect::new(inner.x, y as u16, side_w + 1, 1),
                0,
                i,
                || {
                    let f = format_compact;
                    (
                        it.label.clone(),
                        vec![
                            format!("{} → {}", f(it.start), f(it.end)),
                            format!("change {}", f(it.end - it.start)),
                        ],
                    )
                },
            );
            put_str(buf, x, y as u16, cv_x, &s, Style::default().fg(c));
            buf[(cv_x - 1, y as u16)]
                .set_char('●')
                .set_style(Style::default().fg(c));
        }
        for (k, y) in r_rows.into_iter().filter(|(_, y)| visible(y)) {
            let (i, it) = items[k];
            let c = self.color_of(i, it);
            let s = format!("{} {}", format_compact(it.end), it.label);
            let s: String = s.chars().take(side_w as usize).collect();
            let x = cv_x + cv_w + 1;

            crate::interact::record_point(
                Rect::new(cv_x + cv_w, y as u16, side_w + 1, 1),
                0,
                i,
                || {
                    let f = format_compact;
                    (
                        it.label.clone(),
                        vec![
                            format!("{} → {}", f(it.start), f(it.end)),
                            format!("change {}", f(it.end - it.start)),
                        ],
                    )
                },
            );
            buf[(cv_x + cv_w, y as u16)]
                .set_char('●')
                .set_style(Style::default().fg(c));
            put_str(buf, x, y as u16, inner.right(), &s, Style::default().fg(c));
        }
        if has_titles {
            let bold = text.add_modifier(Modifier::BOLD);
            put_str(buf, inner.x, inner.y, cv_x, &self.titles.0, bold);
            let w = self.titles.1.chars().count() as u16;
            put_str(
                buf,
                inner.right().saturating_sub(w),
                inner.y,
                inner.right(),
                &self.titles.1,
                bold,
            );
            for x in cv_x..cv_x + cv_w {
                buf[(x, inner.y)].set_char('┈').set_style(dim);
            }
        }
    }
}

widget_impls!(SlopeChart);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for SlopeChart {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(c: &SlopeChart, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    fn items() -> Vec<SlopeItem> {
        vec![
            SlopeItem::new("up", 10.0, 30.0),
            SlopeItem::new("down", 25.0, 5.0),
            SlopeItem::new("flat", 15.0, 15.0),
        ]
    }

    #[test]
    fn shows_labels_titles_and_trend_colors() {
        let b = render(&SlopeChart::new(items()).titles("2020", "2024"), 60, 12);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains("up") && t.contains("2024"));
        assert!(b.content().iter().any(|c| c.fg == Color::Green));
        assert!(b.content().iter().any(|c| c.fg == Color::Red));
    }

    #[test]
    fn explicit_color_and_palette_mode() {
        let c = SlopeChart::new([SlopeItem::new("a", 1.0, 2.0).color(Color::Magenta)]);
        assert!(render(&c, 50, 8)
            .content()
            .iter()
            .any(|c| c.fg == Color::Magenta));
        let c = SlopeChart::new(items()).trend_colors(false);
        assert!(!render(&c, 50, 12)
            .content()
            .iter()
            .any(|c| c.fg == Color::Green));
    }

    #[test]
    fn colliding_labels_get_separate_rows() {
        let c = SlopeChart::new([
            SlopeItem::new("aa", 10.0, 10.0),
            SlopeItem::new("bb", 10.0, 10.0),
            SlopeItem::new("cc", 10.0, 10.0),
        ]);
        let b = render(&c, 40, 8);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains("aa") && t.contains("bb") && t.contains("cc"));
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let c = SlopeChart::new([
            SlopeItem::new("n", f64::NAN, 1.0),
            SlopeItem::new("i", f64::INFINITY, f64::NEG_INFINITY),
            SlopeItem::new("big", -1e300, 1e300),
        ]);
        for (w, h) in [(0, 0), (1, 1), (16, 3), (40, 2), (200, 60)] {
            render(&c, w, h);
            render(&SlopeChart::new(items()).titles("a", "b"), w, h);
        }
        render(&SlopeChart::new([]), 40, 10);
    }
}
