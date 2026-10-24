//! Lollipop / dumbbell chart: horizontal stems ending in a dot, or two dots
//! joined by a bar when an item has a start value.

use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::{darken, dim_text, format_compact, format_tick, nice_bounds, nice_ticks, pick};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Block,
};

/// One row of a [`LollipopChart`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LollipopItem {
    /// Row label.
    pub label: String,
    /// Value (the dot).
    pub value: f64,
    /// Start of a dumbbell; `None` draws a stem from the baseline.
    pub from: Option<f64>,
    /// Explicit color.
    pub color: Option<Color>,
}

impl LollipopItem {
    /// Create an item.
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            from: None,
            color: None,
        }
    }

    /// Make this a dumbbell from `from` to `value`.
    pub fn from(mut self, from: f64) -> Self {
        self.from = Some(from);
        self
    }

    /// Explicit color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

/// Lollipop / dumbbell chart widget.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{LollipopChart, LollipopItem};
///
/// let chart = LollipopChart::new([LollipopItem::new("a", 3.0), LollipopItem::new("b", 5.0).from(1.0)]);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct LollipopChart {
    items: Vec<LollipopItem>,
    baseline: f64,
    show_values: bool,
    head: char,
    sorted: bool,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl LollipopChart {
    /// Create a chart.
    pub fn new(items: impl IntoIterator<Item = LollipopItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            baseline: 0.0,
            show_values: true,
            head: '●',
            sorted: false,
            theme: None,
            base: None,
            block: None,
        }
    }

    setter!(
        /// Value the stems start from (default 0).
        baseline: f64
    );
    setter!(
        /// Print the value after each dot (default on).
        show_values: bool
    );
    setter!(
        /// Dot glyph (default `●`).
        head: char
    );
    setter!(
        /// Sort rows by value, largest first.
        sorted: bool
    );
    theme_setter!(
        /// Apply a [`Theme`]; explicit item colors win.
        theme
    );
    base_style_setter!(base);
    block_setter!(block, 'static);

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 10 || inner.height < 2 {
            return;
        }
        let mut rows: Vec<&LollipopItem> = self
            .items
            .iter()
            .filter(|i| i.value.is_finite() && i.from.is_none_or(f64::is_finite))
            .collect();
        if rows.is_empty() {
            return;
        }
        if self.sorted {
            rows.sort_by(|a, b| b.value.total_cmp(&a.value));
        }
        let (mut lo, mut hi) = (self.baseline, self.baseline);
        for r in &rows {
            for v in [Some(r.value), r.from].into_iter().flatten() {
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        let (lo, hi) = nice_bounds(lo, hi, 5);
        let span = (hi - lo).max(f64::EPSILON);
        let (dim, text) = dim_text(&self.theme);

        let lw = rows
            .iter()
            .map(|r| r.label.chars().count())
            .max()
            .unwrap_or(0)
            .min(inner.width as usize / 3) as u16;
        let vw: u16 = if self.show_values { 7 } else { 1 };
        let plot_x = inner.x + lw + 1;
        let plot_w = inner.width.saturating_sub(lw + 1 + vw);
        let plot_h = inner.height - 1;
        if plot_w < 4 {
            return;
        }
        let px = |v: f64| {
            plot_x + (((v - lo) / span).clamp(0.0, 1.0) * (plot_w - 1) as f64).round() as u16
        };

        // grid + axis
        let ticks = nice_ticks(lo, hi, (plot_w as usize / 10).max(2));
        let step = if ticks.len() >= 2 {
            ticks[1] - ticks[0]
        } else {
            1.0
        };
        for &t in &ticks {
            let x = px(t);
            for y in inner.y..inner.y + plot_h {
                buf[(x, y)].set_char('┊').set_style(dim);
            }
            let l = format_tick(t, step);
            let lx = x.saturating_sub(l.chars().count() as u16 / 2).max(plot_x);
            put_str(buf, lx, inner.bottom() - 1, inner.right(), &l, text);
        }
        let bx = px(self.baseline);
        for y in inner.y..inner.y + plot_h {
            buf[(bx, y)]
                .set_char('│')
                .set_style(dim.add_modifier(Modifier::BOLD));
        }

        let n = rows.len() as u16;
        let step_y = if plot_h >= n * 2 { 2 } else { 1 };
        let used = (n * step_y).min(plot_h);
        let y0 = inner.y + (plot_h - used) / 2;
        for (i, r) in rows.iter().enumerate() {
            let y = y0 + i as u16 * step_y;
            if y >= inner.y + plot_h {
                break;
            }
            crate::interact::record_point(Rect::new(inner.x, y, inner.width, 1), 0, i, || {
                (r.label.clone(), vec![crate::util::format_compact(r.value)])
            });
            let c = r.color.unwrap_or_else(|| pick(&self.theme, 0));
            let stem = Style::default().fg(darken(c, 0.25));
            let label: String = r.label.chars().take(lw as usize).collect();
            let lx = inner.x + lw.saturating_sub(label.chars().count() as u16);
            put_str(buf, lx, y, inner.x + lw, &label, text);
            let start = r.from.unwrap_or(self.baseline);
            let (a, b) = (px(start).min(px(r.value)), px(start).max(px(r.value)));
            for x in a..=b {
                buf[(x, y)].set_char('━').set_style(stem);
            }
            if r.from.is_some() {
                buf[(px(start), y)]
                    .set_char(self.head)
                    .set_style(Style::default().fg(darken(c, 0.45)));
            }
            buf[(px(r.value), y)]
                .set_char(self.head)
                .set_style(Style::default().fg(c).add_modifier(Modifier::BOLD));
            if self.show_values {
                let vx = px(r.value).max(px(start)) + 2;
                put_str(buf, vx, y, inner.right(), &format_compact(r.value), text);
            }
        }
    }
}

widget_impls!(LollipopChart);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for LollipopChart {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(c: &LollipopChart, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    fn txt(b: &Buffer) -> String {
        b.content().iter().map(|c| c.symbol()).collect()
    }

    fn items() -> Vec<LollipopItem> {
        vec![
            LollipopItem::new("alpha", 10.0),
            LollipopItem::new("beta", 25.0),
            LollipopItem::new("gamma", -5.0),
        ]
    }

    #[test]
    fn draws_labels_heads_and_values() {
        let t = txt(&render(&LollipopChart::new(items()), 60, 10));
        assert!(t.contains("alpha") && t.contains('●') && t.contains("25"));
    }

    #[test]
    fn values_toggle_and_custom_head() {
        let c = LollipopChart::new(items()).show_values(false).head('◆');
        let t = txt(&render(&c, 60, 10));
        assert!(t.contains('◆') && !t.contains("25"));
    }

    #[test]
    fn dumbbell_draws_both_ends() {
        let c = LollipopChart::new([LollipopItem::new("x", 20.0).from(5.0).color(Color::Red)]);
        let b = render(&c, 60, 4);
        assert!(b.content().iter().filter(|c| c.symbol() == "●").count() >= 2);
    }

    #[test]
    fn sorted_orders_largest_first() {
        let c = LollipopChart::new(items()).sorted(true);
        let t = txt(&render(&c, 60, 10));
        assert!(t.find("beta").unwrap() < t.find("alpha").unwrap());
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let c = LollipopChart::new([
            LollipopItem::new("n", f64::NAN),
            LollipopItem::new("i", f64::INFINITY).from(f64::NAN),
            LollipopItem::new("big", 1e300),
        ]);
        for (w, h) in [(0, 0), (1, 1), (10, 2), (40, 3), (200, 60)] {
            render(&c, w, h);
            render(&LollipopChart::new(items()), w, h);
        }
        render(&LollipopChart::new([]), 40, 10);
    }
}
