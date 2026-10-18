//! Calendar heatmap: one cell per day laid out in week columns, GitHub style.

use crate::heatmap::Gradient;
use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::timeaxis::civil_from_days;
use crate::util::dim_text;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Calendar heatmap widget: consecutive daily values starting at `start_day`.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{CalendarHeatmap, days_from_civil};
///
/// let chart = CalendarHeatmap::new(days_from_civil(2024, 1, 1), (0..120).map(|i| (i % 7) as f64));
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct CalendarHeatmap {
    start_day: i64,
    values: Vec<f64>,
    gradient: Gradient,
    levels: Option<usize>,
    monday_first: bool,
    show_labels: bool,
    legend: bool,
    cell: char,
    bounds: Option<(f64, f64)>,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl CalendarHeatmap {
    /// `start_day` is days since 1970-01-01 (see [`days_from_civil`](crate::days_from_civil));
    /// `values[i]` is the value of day `start_day + i`. Non-finite values draw as empty.
    pub fn new(start_day: i64, values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            start_day,
            values: values.into_iter().collect(),
            gradient: Gradient::Custom(vec![
                (28, 33, 40),
                (14, 68, 41),
                (0, 109, 50),
                (38, 166, 65),
                (57, 211, 83),
            ]),
            levels: Some(5),
            monday_first: true,
            show_labels: true,
            legend: true,
            cell: '■',
            bounds: None,
            theme: None,
            base: None,
            block: None,
        }
    }

    setter!(
        /// Color gradient (default GitHub greens).
        gradient: Gradient
    );
    opt_setter!(
        /// Quantise into this many discrete levels (default 5); use
        /// [`continuous`](Self::continuous) for a smooth gradient.
        levels: usize
    );
    setter!(
        /// Weeks start on Monday (default) instead of Sunday.
        monday_first: bool
    );
    setter!(
        /// Month and weekday labels (default on).
        show_labels: bool
    );
    setter!(
        /// "Less … More" legend on the bottom row (default on, needs height 10+).
        legend: bool
    );
    setter!(
        /// Cell glyph (default `■`).
        cell: char
    );
    theme_setter!(
        /// Apply a [`Theme`] (labels and empty cells).
        theme
    );
    base_style_setter!(base);
    block_setter!(block, 'static);

    /// Fixed `(min, max)` value range instead of the data range.
    pub fn bounds(mut self, min: f64, max: f64) -> Self {
        self.bounds = Some((min, max));
        self
    }

    /// Smooth gradient instead of discrete levels.
    pub fn continuous(mut self) -> Self {
        self.levels = None;
        self
    }

    fn color_for(&self, v: f64, lo: f64, hi: f64) -> Color {
        let t = if hi > lo {
            ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let t = match self.levels {
            Some(n) if n >= 2 => (t * (n - 1) as f64).round() / (n - 1) as f64,
            _ => t,
        };
        self.gradient.color_at(t)
    }

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 6 || inner.height < 7 || self.values.is_empty() {
            return;
        }
        let (dim, text) = dim_text(&self.theme);
        let (mut lo, mut hi) = self.bounds.unwrap_or((f64::MAX, f64::MIN));
        if self.bounds.is_none() {
            for v in self.values.iter().filter(|v| v.is_finite()) {
                lo = lo.min(*v);
                hi = hi.max(*v);
            }
        }
        if lo > hi {
            lo = 0.0;
            hi = 0.0;
        }

        // weekday of the first day, Monday = 0
        let wd = |d: i64| {
            let m = (d + 3).rem_euclid(7);
            if self.monday_first {
                m
            } else {
                (m + 1) % 7
            }
        };
        let off = wd(self.start_day) as usize;
        let weeks = (off + self.values.len()).div_ceil(7);

        let label_w: u16 = if self.show_labels { 4 } else { 0 };
        let top = inner.y + u16::from(self.show_labels);
        let legend = self.legend && inner.height >= 10;
        let grid_h_avail = inner.height - u16::from(self.show_labels) - u16::from(legend);
        if grid_h_avail < 7 {
            return;
        }
        // widen / heighten cells when the whole year fits with room to spare
        let cw = (((inner.width - label_w) as usize / weeks.max(1)).clamp(2, 4)) as u16;
        let ch = (grid_h_avail / 7).clamp(1, 2);
        let max_weeks = ((inner.width - label_w) / cw) as usize;
        if max_weeks == 0 {
            return;
        }
        // show the most recent weeks when the range does not fit
        let first_week = weeks.saturating_sub(max_weeks);
        let x0 = inner.x + label_w;

        if self.show_labels {
            let names = if self.monday_first {
                [(0, "Mon"), (2, "Wed"), (4, "Fri")]
            } else {
                [(1, "Mon"), (3, "Wed"), (5, "Fri")]
            };
            for (row, n) in names {
                put_str(buf, inner.x, top + row * ch, x0, n, dim);
            }
        }
        let mut last_label_end = 0u16;
        for w in first_week..weeks {
            let x = x0 + (w - first_week) as u16 * cw;
            for r in 0..7usize {
                let idx = (w * 7 + r) as i64 - off as i64;
                if idx < 0 || idx as usize >= self.values.len() {
                    continue;
                }
                let v = self.values[idx as usize];
                let st = if v.is_finite() {
                    Style::default().fg(self.color_for(v, lo, hi))
                } else {
                    dim
                };
                let glyph = if v.is_finite() { self.cell } else { '·' };
                for dy in 0..ch {
                    buf[(x, top + r as u16 * ch + dy)]
                        .set_char(glyph)
                        .set_style(st);
                }
                // month label above the week containing the 1st of a month
                if self.show_labels {
                    let (_, m, d) = civil_from_days(self.start_day + idx);
                    if d == 1 && r < 7 && x >= last_label_end {
                        let name = MONTHS[(m as usize - 1) % 12];
                        put_str(buf, x, inner.y, inner.right(), name, text);
                        last_label_end = x + 4;
                    }
                }
            }
        }
        if legend {
            let y = inner.bottom() - 1;
            let n = self.levels.unwrap_or(5).clamp(2, 9);
            let w = (n * 2 + 11) as u16;
            let mut x = inner.right().saturating_sub(w).max(inner.x);
            put_str(buf, x, y, inner.right(), "Less ", dim);
            x += 5;
            for i in 0..n {
                let c = self.gradient.color_at(i as f64 / (n - 1) as f64);
                if x < inner.right() {
                    buf[(x, y)]
                        .set_char(self.cell)
                        .set_style(Style::default().fg(c));
                }
                x += 2;
            }
            put_str(buf, x, y, inner.right(), "More", dim);
        }
    }
}

widget_impls!(CalendarHeatmap);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for CalendarHeatmap {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::days_from_civil;
    use ratatui::widgets::Widget;

    fn render(c: &CalendarHeatmap, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    fn year() -> CalendarHeatmap {
        CalendarHeatmap::new(
            days_from_civil(2024, 1, 1),
            (0..366).map(|i| (i % 9) as f64),
        )
    }

    #[test]
    fn shows_months_weekdays_and_cells() {
        let b = render(&year(), 120, 12);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains("Jan") && t.contains("Mon") && t.contains('■'));
        assert!(t.contains("Less") && t.contains("More"));
    }

    #[test]
    fn jan_first_2024_is_monday_row_zero() {
        // 2024-01-01 was a Monday: first cell lands on the first grid row.
        let b = render(&year(), 120, 12);
        assert_eq!(b[(4, 1)].symbol(), "■");
    }

    #[test]
    fn sunday_first_shifts_rows() {
        let b = render(&year().monday_first(false), 120, 12);
        assert_eq!(b[(4, 1)].symbol(), " ");
        assert_eq!(b[(4, 2)].symbol(), "■");
    }

    #[test]
    fn levels_quantise_colors() {
        let colors = |c: &CalendarHeatmap| {
            let mut v: Vec<_> = render(c, 120, 12)
                .content()
                .iter()
                .filter(|c| c.symbol() == "■")
                .map(|c| format!("{:?}", c.fg))
                .collect();
            v.sort();
            v.dedup();
            v.len()
        };
        assert!(colors(&year().levels(3)) < colors(&year().continuous()));
    }

    #[test]
    fn narrow_area_shows_latest_weeks() {
        let b = render(&year(), 30, 12);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains("Dec") && !t.contains("Feb"));
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let c = CalendarHeatmap::new(-5, [f64::NAN, f64::INFINITY, 1e300, -1e300, 0.0]);
        for (w, h) in [(0, 0), (1, 1), (5, 7), (12, 12), (40, 9), (300, 80)] {
            render(&c, w, h);
            render(&year(), w, h);
        }
        render(&CalendarHeatmap::new(0, []), 40, 12);
        render(
            &year().show_labels(false).legend(false).bounds(0.0, 1.0),
            60,
            8,
        );
    }
}
