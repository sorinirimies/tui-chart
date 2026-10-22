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
    zero_empty: bool,
    cell: char,
    bounds: Option<(f64, f64)>,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl CalendarHeatmap {
    /// `start_day` is days since 1970-01-01 (see [`days_from_civil`](crate::days_from_civil));
    /// `values[i]` is the value of day `start_day + i`. `NaN` draws as a missing
    /// day (`·`); `±inf` clamp to the top / bottom of the range.
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
            zero_empty: false,
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
        /// Shows a 9-step ramp in [`continuous`](Self::continuous) mode.
        legend: bool
    );
    setter!(
        /// Draw exactly-zero days in the dim theme colour instead of the lowest
        /// gradient colour, so "no activity" is distinct from level 0 (default off).
        zero_empty: bool
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

    /// Fixed `(min, max)` value range instead of the data range. Swapped if
    /// inverted; ignored if either end is not finite.
    pub fn bounds(mut self, min: f64, max: f64) -> Self {
        if min.is_finite() && max.is_finite() {
            self.bounds = Some((min.min(max), min.max(max)));
        }
        self
    }

    /// Smooth gradient instead of discrete levels.
    pub fn continuous(mut self) -> Self {
        self.levels = None;
        self
    }

    fn color_for(&self, v: f64, lo: f64, hi: f64) -> Color {
        let v = if v == f64::INFINITY {
            hi
        } else if v == f64::NEG_INFINITY {
            lo
        } else {
            v
        };
        let t = if hi > lo {
            ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
        } else if lo <= 0.0 {
            // flat data: all-zero is "lowest", anything else sits mid-scale
            0.0
        } else {
            0.5
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

        // labels need one extra row; drop them (not the grid) when height is tight
        let labels = self.show_labels && inner.height >= 8;
        let label_w: u16 = if labels { 4 } else { 0 };
        let top = inner.y + u16::from(labels);
        let legend = self.legend && inner.height >= 10;
        let grid_h_avail = inner.height - u16::from(labels) - u16::from(legend);
        if grid_h_avail < 7 {
            return;
        }
        // widen / heighten cells when the whole year fits with room to spare
        let cw = (((inner.width - label_w) as usize / weeks.max(1)).clamp(2, 4)) as u16;
        // cells are `cw - 1` glyphs wide (min 1) with a one-column gap
        let gw = (cw - 1).max(1);
        let ch = (grid_h_avail / 7).clamp(1, 2);
        let max_weeks = ((inner.width - label_w) / cw) as usize;
        if max_weeks == 0 {
            return;
        }
        // show the most recent weeks when the range does not fit
        let first_week = weeks.saturating_sub(max_weeks);
        let x0 = inner.x + label_w;

        if labels {
            let names = if self.monday_first {
                [(0, "Mon"), (2, "Wed"), (4, "Fri")]
            } else {
                [(1, "Mon"), (3, "Wed"), (5, "Fri")]
            };
            for (row, n) in names {
                put_str(buf, inner.x, top + row * ch, x0, n, dim);
            }
        }
        let len = self.values.len() as i64;
        // month whose 1st falls in week `w`, if any
        let month_start = |w: usize| -> Option<u32> {
            (0..7i64).find_map(|r| {
                let idx = (w * 7) as i64 + r - off as i64;
                if idx < 0 || idx >= len {
                    return None;
                }
                let (_, m, d) = civil_from_days(self.start_day + idx);
                (d == 1).then_some(m)
            })
        };
        let label_span = 4usize.div_ceil(cw as usize);
        let mut last_label_end = 0u16;
        for w in first_week..weeks {
            let x = x0 + (w - first_week) as u16 * cw;
            for r in 0..7usize {
                let idx = (w * 7 + r) as i64 - off as i64;
                if idx < 0 || idx >= len {
                    continue;
                }
                let v = self.values[idx as usize];
                let missing = v.is_nan();
                let st = if missing || (self.zero_empty && v == 0.0) {
                    dim
                } else {
                    Style::default().fg(self.color_for(v, lo, hi))
                };
                let (glyph, width) = if missing { ('·', 1) } else { (self.cell, gw) };
                for dy in 0..ch {
                    for dx in 0..width {
                        buf[(x + dx, top + r as u16 * ch + dy)]
                            .set_char(glyph)
                            .set_style(st);
                    }
                }
            }
            if labels {
                // label the week holding a 1st; on the first visible week fall
                // back to its month unless the next label would collide
                let month = month_start(w).or_else(|| {
                    if w != first_week {
                        return None;
                    }
                    let crowded =
                        (1..=label_span).any(|k| w + k < weeks && month_start(w + k).is_some());
                    let first_idx = (w * 7).saturating_sub(off) as i64;
                    (!crowded).then(|| civil_from_days(self.start_day + first_idx).1)
                });
                if let Some(m) = month {
                    if x >= last_label_end {
                        let name = MONTHS[(m as usize - 1) % 12];
                        put_str(buf, x, inner.y, inner.right(), name, text);
                        last_label_end = x + 4;
                    }
                }
            }
        }
        if legend {
            let y = inner.bottom() - 1;
            let n = self.levels.unwrap_or(9).clamp(2, 9);
            let w = (n * 2 + 9) as u16;
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

    fn text(b: &Buffer) -> String {
        b.content().iter().map(|c| c.symbol()).collect()
    }

    fn row(b: &Buffer, y: u16) -> String {
        (0..b.area.width).map(|x| b[(x, y)].symbol()).collect()
    }

    #[test]
    fn height_seven_drops_labels_but_draws_grid() {
        let b = render(&year(), 120, 7);
        let t = text(&b);
        assert!(t.contains('■') && !t.contains("Jan") && !t.contains("Mon"));
    }

    #[test]
    fn month_labels_positioned_above_first_of_month() {
        // Jan 1 2024 is a Monday: week 0 -> x = 4; Feb 1 is a Thursday in week 4.
        let b = render(&year(), 120, 12);
        let top = row(&b, 0);
        assert_eq!(&top[4..7], "Jan");
        let feb = top.find("Feb").expect("Feb label");
        assert!(feb > 4 && top.find("Mar").unwrap() > feb);
    }

    #[test]
    fn mid_month_start_labels_first_column() {
        let c = CalendarHeatmap::new(days_from_civil(2024, 1, 20), (0..120).map(|i| i as f64));
        let top = row(&render(&c, 120, 12), 0);
        assert_eq!(&top[4..7], "Jan");
        assert!(top.contains("Feb"));
    }

    #[test]
    fn truncated_view_labels_left_edge_month() {
        let top = row(&render(&year(), 30, 12), 0);
        assert!(top.contains("Dec"));
        assert!(
            top[4..8].trim().len() >= 3,
            "left edge month unlabeled: {top:?}"
        );
    }

    #[test]
    fn labels_do_not_overlap() {
        let top = row(&render(&year(), 60, 12), 0);
        for m in MONTHS {
            assert!(top.matches(m).count() <= 1, "{top:?}");
        }
    }

    #[test]
    fn flat_data_is_not_max_colour() {
        let max = Color::Rgb(57, 211, 83);
        let fg = |vals: Vec<f64>| {
            let c = CalendarHeatmap::new(days_from_civil(2024, 1, 1), vals);
            render(&c, 40, 12)[(4, 1)].fg
        };
        assert_ne!(fg(vec![5.0; 30]), max);
        assert_eq!(fg(vec![0.0; 30]), Color::Rgb(28, 33, 40));
    }

    #[test]
    fn inverted_and_nan_bounds_are_normalised() {
        let a = render(&year().bounds(8.0, 0.0), 120, 12);
        let b = render(&year().bounds(0.0, 8.0), 120, 12);
        assert_eq!(a, b);
        let c = render(&year().bounds(f64::NAN, 1.0), 120, 12);
        assert_eq!(c, render(&year(), 120, 12));
    }

    #[test]
    fn nan_is_missing_and_inf_clamps() {
        let c = CalendarHeatmap::new(
            days_from_civil(2024, 1, 1),
            [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, 10.0],
        );
        let b = render(&c, 40, 12);
        assert_eq!(b[(4, 1)].symbol(), "·");
        assert_eq!(b[(4, 2)].fg, Color::Rgb(57, 211, 83));
        assert_eq!(b[(4, 3)].fg, Color::Rgb(28, 33, 40));
    }

    #[test]
    fn zero_empty_uses_dim_style() {
        let vals = [0.0, 5.0, 10.0];
        let mk = |z| CalendarHeatmap::new(days_from_civil(2024, 1, 1), vals).zero_empty(z);
        let off = render(&mk(false), 40, 12)[(4, 1)].fg;
        let on = render(&mk(true), 40, 12)[(4, 1)].fg;
        assert_ne!(off, on);
    }

    #[test]
    fn wide_cells_fill_multiple_columns() {
        // 10 weeks in 60 cols -> cw = 4, glyphs 3 wide
        let c = CalendarHeatmap::new(days_from_civil(2024, 1, 1), (0..70).map(|i| i as f64));
        let b = render(&c, 60, 12);
        assert!((4..7).all(|x| b[(x, 1)].symbol() == "■"));
        assert_eq!(b[(7, 1)].symbol(), " ");
    }

    #[test]
    fn tall_cells_use_two_rows() {
        let b = render(&year(), 120, 18);
        assert_eq!(b[(4, 1)].symbol(), "■");
        assert_eq!(b[(4, 2)].symbol(), "■");
    }

    #[test]
    fn legend_sizes_follow_mode() {
        let swatches = |c: &CalendarHeatmap| {
            let b = render(c, 120, 12);
            row(&b, 11).matches('■').count()
        };
        assert_eq!(swatches(&year()), 5);
        assert_eq!(swatches(&year().continuous()), 9);
        assert_eq!(swatches(&year().levels(3)), 3);
        let t = row(&render(&year(), 120, 12), 11);
        assert!(t.trim_end().ends_with("More") && t.contains("Less"));
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
