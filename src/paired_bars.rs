//! PairedBars chart: two bars per measure, "before" against "after", each measure on its own scale.

use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::darken;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// One measure: a value before and a value after.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PairedBarsItem {
    /// Measure label.
    pub label: String,
    /// The first value (the reference).
    pub before: f64,
    /// The second value.
    pub after: f64,
    /// Printed after both values, e.g. `"%"`.
    pub unit: String,
}

impl PairedBarsItem {
    /// Create a measure without a unit.
    pub fn new(label: impl Into<String>, before: f64, after: f64) -> Self {
        Self {
            label: label.into(),
            before,
            after,
            unit: String::new(),
        }
    }

    /// The unit printed after both values.
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }
}

/// How the two bars of a measure sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PairedBarsLayout {
    /// The label, then both bars on one line (2 rows per measure).
    #[default]
    Inline,
    /// The label, then one bar above the other (3 rows per measure).
    Stacked,
}

/// PairedBars chart widget: for every measure two bars on one scale `0..max(before, after)`,
/// so the scales may differ between measures (mixed units work). An optional legend names the
/// two series.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{PairedBars, PairedBarsItem};
///
/// let chart = PairedBars::new([
///     PairedBarsItem::new("mean ΔE", 3.2, 1.8),
///     PairedBarsItem::new("over ΔE 7", 12.0, 4.5).unit("%"),
/// ])
/// .legend("46 shipped", "286 in use");
/// let area = Rect::new(0, 0, 50, 8);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone, Default)]
pub struct PairedBars {
    items: Vec<PairedBarsItem>,
    layout: PairedBarsLayout,
    legend: Option<(String, String)>,
    decimals: usize,
    max_bar: Option<u16>,
    before_color: Option<Color>,
    after_color: Option<Color>,
    label_color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

fn fin(v: f64) -> f64 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

impl PairedBars {
    /// Create a paired_bars chart (one decimal in the values).
    pub fn new(items: impl IntoIterator<Item = PairedBarsItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            decimals: 1,
            ..Self::default()
        }
    }

    /// Where the bars of a measure sit (default: [`PairedBarsLayout::Inline`]).
    pub fn layout(mut self, l: PairedBarsLayout) -> Self {
        self.layout = l;
        self
    }

    /// A legend line under the measures, naming the series (drawn when a row is left over).
    pub fn legend(mut self, before: impl Into<String>, after: impl Into<String>) -> Self {
        self.legend = Some((before.into(), after.into()));
        self
    }

    /// Decimals of the printed values (default 1).
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }

    /// The longest a bar may be, in cells (default: as wide as the area allows).
    pub fn max_bar_width(mut self, w: u16) -> Self {
        self.max_bar = Some(w);
        self
    }

    /// Color of the "before" bars.
    pub fn before_color(mut self, c: Color) -> Self {
        self.before_color = Some(c);
        self
    }

    /// Color of the measure labels and legend names (default: the theme's).
    pub fn label_color(mut self, c: Color) -> Self {
        self.label_color = Some(c);
        self
    }

    /// Color of the "after" bars.
    pub fn after_color(mut self, c: Color) -> Self {
        self.after_color = Some(c);
        self
    }

    theme_setter!(
        /// Apply a [`Theme`]; explicit colors win.
        theme
    );

    base_style_setter!(base);

    block_setter!(block, 'static);

    /// The height one measure takes.
    fn row_height(&self) -> u16 {
        match self.layout {
            PairedBarsLayout::Inline => 2,
            PairedBarsLayout::Stacked => 3,
        }
    }

    fn value(&self, it: &PairedBarsItem, v: f64) -> String {
        format!("{:.*}{}", self.decimals, fin(v), it.unit)
    }

    /// Report the before / after bars of `it` (`at` holds their `(x, y)`).
    fn record_pair(&self, it: &PairedBarsItem, ii: usize, at: [(u16, u16); 2], w: u16) {
        for (k, (x, y)) in at.into_iter().enumerate() {
            let v = if k == 0 { it.before } else { it.after };
            crate::interact::record_point(Rect::new(x, y, w, 1), k, ii, || {
                let side = if k == 0 { "before" } else { "after" };
                (
                    it.label.clone(),
                    vec![format!("{side}: {}", self.value(it, v))],
                )
            });
        }
    }

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 8 || inner.height == 0 || self.items.is_empty() {
            return;
        }
        let th = self.theme.as_ref();
        let text = th.map_or(Style::default(), |t| t.label_style());
        let text = self.label_color.map_or(text, |c| text.fg(c));
        let accent = th.map_or(Color::Cyan, |t| t.accent);
        let muted = th.map_or(Color::DarkGray, |t| darken(t.foreground, 0.45));
        let before = self.before_color.unwrap_or(muted);
        let after = self.after_color.unwrap_or(accent);
        let rh = self.row_height();
        let wide = self
            .items
            .iter()
            .map(|i| self.value(i, i.before).chars().count());
        let vb = wide.max().unwrap_or(0) as u16 + 1;
        let wide = self
            .items
            .iter()
            .map(|i| self.value(i, i.after).chars().count());
        let va = wide.max().unwrap_or(0) as u16 + 1;
        let mut y = inner.y;
        let rows_fit = (inner.height / rh) as usize;
        for (ii, it) in self.items.iter().take(rows_fit.max(1)).enumerate() {
            if y + rh > inner.bottom() {
                break;
            }
            put_str(buf, inner.x, y, inner.right(), &it.label, text);
            let top = fin(it.before).max(fin(it.after)).max(f64::EPSILON);
            let cells = |v: f64, w: u16| -> u16 {
                if fin(v) <= 0.0 {
                    0
                } else {
                    ((fin(v) / top * f64::from(w)).round() as u16).clamp(1, w)
                }
            };
            let (vbs, vas) = (self.value(it, it.before), self.value(it, it.after));
            match self.layout {
                PairedBarsLayout::Inline => {
                    let gap = 2u16;
                    let free = inner.width.saturating_sub(vb + va + gap);
                    let mut w = free / 2;
                    if let Some(m) = self.max_bar {
                        w = w.min(m);
                    }
                    if w == 0 {
                        y += rh;
                        continue;
                    }
                    let by = y + 1;
                    let x1 = inner.x;
                    self.record_pair(it, ii, [(x1, by), (x1 + w + vb + gap, by)], w);
                    bar(buf, x1, by, cells(it.before, w), before);
                    put_str(
                        buf,
                        x1 + w,
                        by,
                        inner.right(),
                        &format!(" {vbs}"),
                        Style::default().fg(before),
                    );
                    let x2 = x1 + w + vb + gap;
                    bar(buf, x2, by, cells(it.after, w), after);
                    put_str(
                        buf,
                        x2 + w,
                        by,
                        inner.right(),
                        &format!(" {vas}"),
                        Style::default().fg(after),
                    );
                }
                PairedBarsLayout::Stacked => {
                    let v = vb.max(va);
                    let mut w = inner.width.saturating_sub(v);
                    if let Some(m) = self.max_bar {
                        w = w.min(m);
                    }
                    if w == 0 {
                        y += rh;
                        continue;
                    }
                    self.record_pair(it, ii, [(inner.x, y + 1), (inner.x, y + 2)], w);
                    bar(buf, inner.x, y + 1, cells(it.before, w), before);
                    put_str(
                        buf,
                        inner.x + w,
                        y + 1,
                        inner.right(),
                        &format!(" {vbs}"),
                        Style::default().fg(before),
                    );
                    bar(buf, inner.x, y + 2, cells(it.after, w), after);
                    put_str(
                        buf,
                        inner.x + w,
                        y + 2,
                        inner.right(),
                        &format!(" {vas}"),
                        Style::default().fg(after),
                    );
                }
            }
            y += rh;
        }
        if let Some((b, a)) = &self.legend {
            if y < inner.bottom() {
                let mut x = inner.x;
                for (name, color) in [(b, before), (a, after)] {
                    put_str(buf, x, y, inner.right(), "■ ", Style::default().fg(color));
                    put_str(buf, x + 2, y, inner.right(), name, text);
                    x += 2 + name.chars().count() as u16 + 3;
                }
            }
        }
    }
}

/// `cells` full blocks from `x`.
fn bar(buf: &mut Buffer, x: u16, y: u16, cells: u16, color: Color) {
    for dx in 0..cells {
        if let Some(c) = buf.cell_mut((x + dx, y)) {
            c.set_char('█').set_fg(color);
        }
    }
}

widget_impls!(PairedBars);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for PairedBars {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(c: &PairedBars, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(a);
        c.render(a, &mut buf);
        buf
    }

    fn text(buf: &Buffer) -> String {
        buf.content().iter().map(|c| c.symbol()).collect()
    }

    fn blocks(buf: &Buffer, color: Color) -> usize {
        buf.content()
            .iter()
            .filter(|c| c.symbol() == "█" && c.fg == color)
            .count()
    }

    #[test]
    fn the_larger_value_gets_the_longer_bar() {
        let c = PairedBars::new([PairedBarsItem::new("mean", 4.0, 1.0)])
            .before_color(Color::Red)
            .after_color(Color::Green);
        let buf = render(&c, 50, 2);
        assert!(blocks(&buf, Color::Red) > blocks(&buf, Color::Green));
        assert!(text(&buf).contains("mean"));
        assert!(text(&buf).contains("4.0") && text(&buf).contains("1.0"));
    }

    #[test]
    fn a_value_above_zero_always_shows_a_bar() {
        let c =
            PairedBars::new([PairedBarsItem::new("m", 1000.0, 0.001)]).after_color(Color::Green);
        assert_eq!(blocks(&render(&c, 50, 2), Color::Green), 1);
        let zero = PairedBars::new([PairedBarsItem::new("m", 5.0, 0.0)]).after_color(Color::Green);
        assert_eq!(blocks(&render(&zero, 50, 2), Color::Green), 0);
    }

    #[test]
    fn each_measure_has_its_own_scale_and_unit() {
        let c = PairedBars::new([
            PairedBarsItem::new("a", 2.0, 1.0),
            PairedBarsItem::new("b", 90.0, 45.0).unit("%"),
        ])
        .before_color(Color::Red);
        let buf = render(&c, 50, 4);
        assert!(text(&buf).contains("90.0%"));
        // both rows have the full bar for their larger value
        let per_row = |y: u16| {
            (0..50)
                .filter(|x| buf[(*x, y)].fg == Color::Red && buf[(*x, y)].symbol() == "█")
                .count()
        };
        assert_eq!(per_row(1), per_row(3));
    }

    #[test]
    fn stacked_puts_the_bars_on_two_rows() {
        let c = PairedBars::new([PairedBarsItem::new("m", 4.0, 2.0)])
            .layout(PairedBarsLayout::Stacked)
            .before_color(Color::Red)
            .after_color(Color::Green);
        let buf = render(&c, 40, 3);
        let row = |y: u16, color: Color| {
            (0..40)
                .filter(|x| buf[(*x, y)].fg == color && buf[(*x, y)].symbol() == "█")
                .count()
        };
        assert!(row(1, Color::Red) > 0 && row(1, Color::Green) == 0);
        assert!(row(2, Color::Green) > 0 && row(2, Color::Red) == 0);
    }

    #[test]
    fn the_legend_needs_a_spare_row() {
        let c = PairedBars::new([PairedBarsItem::new("m", 4.0, 2.0)]).legend("old", "new");
        assert!(text(&render(&c, 40, 3)).contains("new"));
        assert!(!text(&render(&c, 40, 2)).contains("new"));
    }

    #[test]
    fn decimals_and_a_cap_on_the_bar() {
        let c = PairedBars::new([PairedBarsItem::new("m", 4.0, 2.0)])
            .decimals(2)
            .max_bar_width(5)
            .before_color(Color::Red);
        let buf = render(&c, 80, 2);
        assert!(text(&buf).contains("4.00"));
        assert_eq!(blocks(&buf, Color::Red), 5);
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let c = PairedBars::new([PairedBarsItem::new("x", f64::NAN, f64::INFINITY)]);
        for l in [PairedBarsLayout::Inline, PairedBarsLayout::Stacked] {
            let c = c.clone().layout(l).legend("a", "b");
            for (w, h) in [(0, 0), (3, 1), (8, 1), (20, 2), (40, 12)] {
                render(&c, w, h);
            }
        }
        render(&PairedBars::new([]), 20, 3);
        render(
            &PairedBars::new([PairedBarsItem::new("n", -3.0, -1.0)]),
            20,
            3,
        );
    }

    #[test]
    fn labels_take_their_color() {
        let c = PairedBars::new([PairedBarsItem::new("lab", 4.0, 2.0)]).label_color(Color::Magenta);
        let buf = render(&c, 40, 2);
        assert_eq!(buf[(0, 0)].fg, Color::Magenta);
    }

    #[test]
    fn many_measures_are_cut_to_fit() {
        let items = (0..8).map(|i| PairedBarsItem::new(format!("m{i}"), 3.0, 1.0));
        let c = PairedBars::new(items);
        for h in [1, 2, 3, 7, 40] {
            render(&c, 50, h);
        }
    }
}
