//! Hit testing for [`PieChart`](tui_piechart::PieChart).
//!
//! The pie widget lives in `tui-piechart` and knows nothing about
//! [`HitMap`](crate::HitMap). [`Pie`] wraps it: it renders the chart as usual,
//! then renders a second, off-screen copy where every slice has a unique color
//! and reads the owner of each cell back from that. This works for every
//! [`PieStyle`](tui_piechart::PieStyle) and resolution without duplicating
//! the geometry.

use ratatui::{buffer::Buffer, layout::Rect, style::Color, style::Modifier};
use tui_piechart::{PieChart, PieSlice};

use crate::util::format_compact;

/// A [`PieChart`] that reports hover / click regions and supports hiding
/// slices (see [`Interaction`](crate::Interaction)).
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, style::Color};
/// use tui_chart::{HitMap, Pie, PieChart, PieSlice};
///
/// let chart = PieChart::new(vec![
///     PieSlice::new("Rust", 60.0, Color::Red),
///     PieSlice::new("Go", 40.0, Color::Blue),
/// ]);
/// let area = Rect::new(0, 0, 60, 20);
/// let mut buf = Buffer::empty(area);
/// let hits = HitMap::capture(Pie::new(chart).hidden([1]), area, &mut buf);
/// assert!(!hits.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Pie<'a> {
    chart: PieChart<'a>,
    hidden: Vec<usize>,
}

impl<'a> Pie<'a> {
    /// Wrap a chart.
    pub fn new(chart: PieChart<'a>) -> Self {
        Self {
            chart,
            hidden: Vec::new(),
        }
    }

    /// Hide slices by index; they stay in the legend (dimmed) so they can be
    /// toggled back.
    #[must_use]
    pub fn hidden(mut self, hidden: impl IntoIterator<Item = usize>) -> Self {
        self.hidden = hidden.into_iter().collect();
        self
    }

    fn slices(&self, color: impl Fn(usize, &PieSlice<'a>) -> Color) -> Vec<PieSlice<'a>> {
        self.chart
            .slice_list()
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let v = if self.hidden.contains(&i) {
                    0.0
                } else {
                    s.value()
                };
                PieSlice::new(s.label(), v, color(i, s))
            })
            .collect()
    }

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let visible = self.chart.clone().slices(self.slices(|_, s| s.color()));
        ratatui::widgets::Widget::render(&visible, area, buf);
        if area.is_empty() {
            return;
        }
        let (pie, legend) = self.chart.areas(area);
        // dim hidden legend entries
        let marks = if crate::interact::recording() || !self.hidden.is_empty() {
            self.owners(area, pie, legend)
        } else {
            return;
        };
        for (i, rect) in &marks.legend {
            if self.hidden.contains(i) {
                for x in rect.left()..rect.right() {
                    let c = &mut buf[(x, rect.y)];
                    c.set_style(
                        ratatui::style::Style::default()
                            .add_modifier(Modifier::DIM | Modifier::CROSSED_OUT),
                    );
                }
            }
            crate::interact::record_legend(*rect, *i, self.chart.slice_list()[*i].label());
        }
        let total: f64 = self
            .chart
            .slice_list()
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.hidden.contains(i))
            .map(|(_, s)| s.value().max(0.0))
            .sum();
        crate::interact::record_grid(
            pie,
            &marks.pie,
            |i| i,
            |i| {
                let s = &self.chart.slice_list()[i];
                let pct = if total > 0.0 {
                    s.value().max(0.0) / total * 100.0
                } else {
                    0.0
                };
                (
                    s.label().to_string(),
                    vec![format_compact(s.value()), format!("{pct:.1}%")],
                )
            },
        );
    }

    /// Render with unique slice colors and read back who owns which cell.
    fn owners(&self, area: Rect, pie: Rect, legend: Option<Rect>) -> Owners {
        let key = |i: usize| Color::Rgb((i >> 8) as u8 + 1, i as u8, 77);
        let sentinel = self.chart.clone().slices(self.slices(|i, _| key(i)));
        let mut scratch = Buffer::empty(area);
        ratatui::widgets::Widget::render(&sentinel, area, &mut scratch);
        let n = self.chart.slice_list().len();
        let owner = |c: &ratatui::buffer::Cell| match c.fg {
            Color::Rgb(h, l, 77) if h >= 1 => {
                let i = usize::from(h - 1) << 8 | usize::from(l);
                (i < n).then_some(i)
            }
            _ => None,
        };
        let mut grid = vec![None; usize::from(pie.width) * usize::from(pie.height)];
        for y in pie.top()..pie.bottom() {
            for x in pie.left()..pie.right() {
                if legend.is_some_and(|l| l.contains((x, y).into())) {
                    continue;
                }
                grid[usize::from(y - pie.y) * usize::from(pie.width) + usize::from(x - pie.x)] =
                    owner(&scratch[(x, y)]);
            }
        }
        let mut rects: Vec<(usize, Rect)> = Vec::new();
        if let Some(l) = legend {
            for y in l.top()..l.bottom() {
                for x in l.left()..l.right() {
                    if let Some(i) = owner(&scratch[(x, y)]) {
                        if rects.iter().all(|(j, _)| *j != i) {
                            let label = self.chart.slice_list()[i].label().chars().count() as u16;
                            let w = (2 + label).min(l.right() - x);
                            rects.push((i, Rect::new(x, y, w, 1)));
                        }
                    }
                }
            }
        }
        Owners {
            pie: grid,
            legend: rects,
        }
    }
}

struct Owners {
    pie: Vec<Option<usize>>,
    legend: Vec<(usize, Rect)>,
}

widget_impls!(Pie<'_>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HitKind, HitMap};

    fn chart() -> PieChart<'static> {
        PieChart::new(vec![
            PieSlice::new("Rust", 60.0, Color::Red),
            PieSlice::new("Go", 30.0, Color::Blue),
            PieSlice::new("C", 10.0, Color::Green),
        ])
    }

    fn capture(pie: Pie<'_>) -> (HitMap, Buffer) {
        let area = Rect::new(0, 0, 70, 22);
        let mut buf = Buffer::empty(area);
        (HitMap::capture(pie, area, &mut buf), buf)
    }

    #[test]
    fn slices_and_legend_are_reported() {
        let (h, _) = capture(Pie::new(chart()));
        for s in 0..3 {
            assert!(h
                .hits()
                .iter()
                .any(|x| x.kind == HitKind::Point && x.series == s));
            assert!(h
                .hits()
                .iter()
                .any(|x| x.kind == HitKind::Legend && x.series == s));
        }
        let big = h
            .hits()
            .iter()
            .find(|x| x.series == 0 && x.kind == HitKind::Point)
            .unwrap();
        assert!(big.lines.iter().any(|l| l.starts_with("60.0")));
    }

    #[test]
    fn hidden_slice_drops_out_and_is_dimmed() {
        let (h, buf) = capture(Pie::new(chart()).hidden([1]));
        assert!(h
            .hits()
            .iter()
            .all(|x| x.series != 1 || x.kind == HitKind::Legend));
        let leg = h.hits().iter().find(|x| x.series == 1).unwrap();
        assert!(buf[(leg.rect.x, leg.rect.y)]
            .modifier
            .contains(Modifier::CROSSED_OUT));
    }

    #[test]
    fn styles_and_donut_work_too() {
        use tui_piechart::PieStyle;
        for style in [
            PieStyle::Solid,
            PieStyle::Segmented,
            PieStyle::Hud,
            PieStyle::Rose,
        ] {
            let (h, _) = capture(Pie::new(chart().pie_style(style).hole(0.4)));
            assert!(
                h.hits().iter().any(|x| x.kind == HitKind::Point),
                "{style:?}"
            );
        }
    }
}
