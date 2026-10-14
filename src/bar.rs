//! Vertical / horizontal, grouped / stacked bar charts.

use crate::axis::Axis;
use crate::layout::{self, text_width};
use crate::util::{format_compact, nice_bounds};
use crate::PALETTE;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

/// Bar direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarOrientation {
    /// Bars grow upwards.
    #[default]
    Vertical,
    /// Bars grow to the right.
    Horizontal,
}

/// How multiple values per group are combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarMode {
    /// Side by side.
    #[default]
    Grouped,
    /// Stacked on top of each other.
    Stacked,
}

/// A labelled group of bars (one value per series).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BarGroup {
    /// Category label.
    pub label: String,
    /// One value per series.
    pub values: Vec<f64>,
}

impl BarGroup {
    /// Create a group.
    pub fn new(label: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            label: label.into(),
            values: values.into_iter().collect(),
        }
    }
}

const BOTTOM_EIGHTHS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const LEFT_EIGHTHS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

/// Bar chart widget.
#[derive(Debug, Clone, Default)]
pub struct BarChart {
    groups: Vec<BarGroup>,
    series_names: Vec<String>,
    colors: Vec<Color>,
    orientation: BarOrientation,
    mode: BarMode,
    bar_width: Option<u16>,
    bar_gap: u16,
    group_gap: Option<u16>,
    show_values: bool,
    max: Option<f64>,
    block: Option<Block<'static>>,
    value_axis: Axis,
    category_axis: Axis,
}

impl BarChart {
    /// Create from groups.
    pub fn new(groups: Vec<BarGroup>) -> Self {
        Self {
            groups,
            value_axis: Axis::new().grid(true),
            ..Default::default()
        }
    }

    /// Names of the series (legend).
    pub fn series_names<I, S>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.series_names = names.into_iter().map(Into::into).collect();
        self
    }

    /// Series colors.
    pub fn colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self
    }

    /// Direction.
    pub fn orientation(mut self, o: BarOrientation) -> Self {
        self.orientation = o;
        self
    }

    /// Grouped or stacked.
    pub fn mode(mut self, m: BarMode) -> Self {
        self.mode = m;
        self
    }

    /// Bar thickness in cells (default: auto-fit 1–8).
    pub fn bar_width(mut self, w: u16) -> Self {
        self.bar_width = Some(w.max(1));
        self
    }

    /// Cells between bars in a group.
    pub fn bar_gap(mut self, g: u16) -> Self {
        self.bar_gap = g;
        self
    }

    /// Cells between groups (default 1).
    pub fn group_gap(mut self, g: u16) -> Self {
        self.group_gap = Some(g);
        self
    }

    /// Print values next to bars.
    pub fn show_values(mut self, on: bool) -> Self {
        self.show_values = on;
        self
    }

    /// Fixed maximum of the value axis.
    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    /// Wrap in a block.
    pub fn block(mut self, b: Block<'static>) -> Self {
        self.block = Some(b);
        self
    }

    /// Value axis (y for vertical, x for horizontal).
    pub fn value_axis(mut self, a: Axis) -> Self {
        self.value_axis = a;
        self
    }

    /// Category axis (title/style only; labels come from groups).
    pub fn category_axis(mut self, a: Axis) -> Self {
        self.category_axis = a;
        self
    }

    fn n_series(&self) -> usize {
        self.groups
            .iter()
            .map(|g| g.values.len())
            .max()
            .unwrap_or(1)
            .max(1)
    }

    fn color(&self, i: usize) -> Color {
        self.colors
            .get(i)
            .copied()
            .unwrap_or(PALETTE[i % PALETTE.len()])
    }

    fn clean(v: f64) -> f64 {
        if v.is_finite() && v > 0.0 {
            v
        } else {
            0.0
        }
    }

    fn group_total(&self, g: &BarGroup) -> f64 {
        match self.mode {
            BarMode::Stacked => g.values.iter().map(|&v| Self::clean(v)).sum(),
            BarMode::Grouped => g.values.iter().map(|&v| Self::clean(v)).fold(0.0, f64::max),
        }
    }

    /// Value-axis bounds `(min, max)`.
    pub fn bounds(&self) -> (f64, f64) {
        if let Some(b) = self.value_axis.bounds {
            return b;
        }
        let max = self.max.unwrap_or_else(|| {
            self.groups
                .iter()
                .map(|g| self.group_total(g))
                .fold(0.0, f64::max)
        });
        if max <= 0.0 {
            return (0.0, 1.0);
        }
        let (_, hi) = nice_bounds(0.0, max, self.value_axis.ticks);
        (0.0, hi)
    }

    fn format_value(&self, v: f64) -> String {
        match self.value_axis.formatter {
            Some(f) => f(v),
            None => format_compact(v),
        }
    }

    /// Segment boundaries (cumulative eighths) per bar for stacked mode.
    fn eighths_for(&self, g: &BarGroup, lo: f64, hi: f64, cells: u16) -> Vec<i32> {
        let scale = cells as f64 * 8.0 / (hi - lo);
        match self.mode {
            BarMode::Stacked => {
                let mut acc = 0.0;
                g.values
                    .iter()
                    .map(|&v| {
                        acc += Self::clean(v);
                        (((acc - lo) * scale).round() as i32).clamp(0, cells as i32 * 8)
                    })
                    .collect()
            }
            BarMode::Grouped => g
                .values
                .iter()
                .map(|&v| {
                    (((Self::clean(v) - lo) * scale).round() as i32).clamp(0, cells as i32 * 8)
                })
                .collect(),
        }
    }

    fn auto_thickness(&self, avail: u16, n_groups: usize, per_group: usize, group_gap: u16) -> u16 {
        if let Some(w) = self.bar_width {
            return w;
        }
        let g = n_groups.max(1) as i32;
        let n = per_group.max(1) as i32;
        let slot = avail as i32 / g;
        let w = (slot - group_gap as i32 - self.bar_gap as i32 * (n - 1)) / n;
        w.clamp(
            1,
            if self.orientation == BarOrientation::Vertical {
                8
            } else {
                3
            },
        ) as u16
    }

    fn render_inner(&self, area: Rect, buf: &mut Buffer) {
        let inner = layout::render_block(&self.block, area, buf);
        match self.orientation {
            BarOrientation::Vertical => self.render_vertical(inner, buf),
            BarOrientation::Horizontal => self.render_horizontal(inner, buf),
        }
    }

    fn render_vertical(&self, inner: Rect, buf: &mut Buffer) {
        let (lo, hi) = self.bounds();
        if hi <= lo {
            return;
        }
        let marks = self.value_axis.tick_marks((lo, hi));
        let yw = marks.iter().map(|(_, l)| text_width(l)).max().unwrap_or(0);
        let has_labels = self.groups.iter().any(|g| !g.label.is_empty());
        let Some(a) = layout::compute(
            inner,
            yw,
            has_labels,
            self.category_axis.title.is_some(),
            self.value_axis.title.is_some(),
        ) else {
            return;
        };
        let plot = a.plot;
        let n = if self.mode == BarMode::Stacked {
            1
        } else {
            self.n_series()
        };
        let gg = self.group_gap.unwrap_or(1);
        let bw = self.auto_thickness(plot.width, self.groups.len(), n, gg);
        let group_w = bw * n as u16 + self.bar_gap * (n as u16 - 1);
        let slot = (group_w + gg) as usize;
        let visible = ((plot.width as usize + gg as usize) / slot).max(1);

        let y_ticks: Vec<(u16, String)> = marks
            .iter()
            .map(|(f, l)| (layout::frac_row(*f, plot.height, 8), l.clone()))
            .collect();
        if self.value_axis.grid {
            let rows: Vec<u16> = y_ticks.iter().map(|t| t.0).collect();
            layout::draw_grid(buf, plot, &[], &rows, self.value_axis.grid_style);
        }

        let mut x_ticks = vec![];
        for (gi, g) in self.groups.iter().take(visible).enumerate() {
            let gx = plot.x + (gi * slot) as u16;
            if gx + group_w > plot.right() {
                break;
            }
            x_ticks.push((gx - plot.x + group_w / 2, g.label.clone()));
            let eighths = self.eighths_for(g, lo, hi, plot.height);
            let bars = if self.mode == BarMode::Stacked {
                1
            } else {
                g.values.len()
            };
            for bi in 0..bars {
                let x = gx + bi as u16 * (bw + self.bar_gap);
                let total = if self.mode == BarMode::Stacked {
                    eighths.last().copied().unwrap_or(0)
                } else {
                    eighths[bi]
                };
                for r in 0..plot.height as i32 {
                    let fill = (total - r * 8).clamp(0, 8) as usize;
                    if fill == 0 {
                        break;
                    }
                    let color = if self.mode == BarMode::Stacked {
                        let p = r * 8 + fill as i32 - 1;
                        let k = eighths
                            .iter()
                            .position(|&c| p < c)
                            .unwrap_or(eighths.len() - 1);
                        self.color(k)
                    } else {
                        self.color(bi)
                    };
                    let y = plot.bottom() - 1 - r as u16;
                    for dx in 0..bw {
                        buf.set_string(
                            x + dx,
                            y,
                            BOTTOM_EIGHTHS[fill].to_string(),
                            Style::default().fg(color),
                        );
                    }
                }
                if self.show_values && total > 0 {
                    let value = if self.mode == BarMode::Stacked {
                        self.group_total(g)
                    } else {
                        g.values[bi]
                    };
                    let text = self.format_value(value);
                    let tw = text_width(&text);
                    let top_cells = (total + 7) / 8;
                    if top_cells < plot.height as i32 && tw <= bw + 1 + self.bar_gap {
                        let y = plot.bottom() - 1 - top_cells as u16;
                        let tx = (x + bw / 2).saturating_sub(tw / 2).max(plot.x);
                        let tx = tx.min(plot.right().saturating_sub(tw));
                        buf.set_string(tx, y, text, Style::default().fg(Color::Gray));
                    }
                }
            }
        }

        layout::draw_axes(buf, &a, self.category_axis.style, self.value_axis.style);
        layout::draw_y_labels(buf, a.y_labels, &y_ticks, self.value_axis.label_style);
        layout::draw_x_labels(buf, a.x_labels, &x_ticks, self.category_axis.label_style);
        layout::draw_titles(
            buf,
            inner,
            &a,
            self.category_axis
                .title
                .as_deref()
                .map(|t| (t, self.category_axis.label_style)),
            self.value_axis
                .title
                .as_deref()
                .map(|t| (t, self.value_axis.label_style)),
        );
        self.draw_legend(buf, plot);
    }

    fn render_horizontal(&self, inner: Rect, buf: &mut Buffer) {
        let (lo, hi) = self.bounds();
        if hi <= lo {
            return;
        }
        let marks = self.value_axis.tick_marks((lo, hi));
        let yw = self
            .groups
            .iter()
            .map(|g| text_width(&g.label))
            .max()
            .unwrap_or(0)
            .min(16);
        let Some(a) = layout::compute(
            inner,
            yw,
            !marks.is_empty(),
            self.value_axis.title.is_some(),
            self.category_axis.title.is_some(),
        ) else {
            return;
        };
        let plot = a.plot;
        let n = if self.mode == BarMode::Stacked {
            1
        } else {
            self.n_series()
        };
        let gg = self
            .group_gap
            .unwrap_or(if plot.height as usize >= self.groups.len() * (n + 1) {
                1
            } else {
                0
            });
        let bh = self.auto_thickness(plot.height, self.groups.len(), n, gg);
        let group_h = bh * n as u16 + self.bar_gap * (n as u16 - 1);
        let slot = (group_h + gg) as usize;
        let visible = ((plot.height as usize + gg as usize) / slot).max(1);

        let x_ticks: Vec<(u16, String)> = marks
            .iter()
            .map(|(f, l)| (layout::frac_col(*f, plot.width, 1), l.clone()))
            .collect();
        if self.value_axis.grid {
            let cols: Vec<u16> = x_ticks.iter().map(|t| t.0).collect();
            layout::draw_grid(buf, plot, &cols, &[], self.value_axis.grid_style);
        }

        let mut y_ticks = vec![];
        for (gi, g) in self.groups.iter().take(visible).enumerate() {
            let gy = plot.y + (gi * slot) as u16;
            if gy + group_h > plot.bottom() {
                break;
            }
            let label: String = g.label.chars().take(yw as usize).collect();
            y_ticks.push((gy - plot.y + group_h / 2, label));
            let eighths = self.eighths_for(g, lo, hi, plot.width);
            let bars = if self.mode == BarMode::Stacked {
                1
            } else {
                g.values.len()
            };
            for bi in 0..bars {
                let y = gy + bi as u16 * (bh + self.bar_gap);
                let total = if self.mode == BarMode::Stacked {
                    eighths.last().copied().unwrap_or(0)
                } else {
                    eighths[bi]
                };
                for c in 0..plot.width as i32 {
                    let fill = (total - c * 8).clamp(0, 8) as usize;
                    if fill == 0 {
                        break;
                    }
                    let color = if self.mode == BarMode::Stacked {
                        let p = c * 8 + fill as i32 - 1;
                        let k = eighths
                            .iter()
                            .position(|&e| p < e)
                            .unwrap_or(eighths.len() - 1);
                        self.color(k)
                    } else {
                        self.color(bi)
                    };
                    for dy in 0..bh {
                        buf.set_string(
                            plot.x + c as u16,
                            y + dy,
                            LEFT_EIGHTHS[fill].to_string(),
                            Style::default().fg(color),
                        );
                    }
                }
                if self.show_values && total > 0 {
                    let value = if self.mode == BarMode::Stacked {
                        self.group_total(g)
                    } else {
                        g.values[bi]
                    };
                    let text = self.format_value(value);
                    let tw = text_width(&text);
                    let end = ((total + 7) / 8) as u16;
                    if end + 1 + tw <= plot.width {
                        buf.set_string(
                            plot.x + end + 1,
                            y + bh / 2,
                            text,
                            Style::default().fg(Color::Gray),
                        );
                    }
                }
            }
        }

        layout::draw_axes(buf, &a, self.value_axis.style, self.category_axis.style);
        layout::draw_y_labels(buf, a.y_labels, &y_ticks, self.category_axis.label_style);
        layout::draw_x_labels(buf, a.x_labels, &x_ticks, self.value_axis.label_style);
        layout::draw_titles(
            buf,
            inner,
            &a,
            self.value_axis
                .title
                .as_deref()
                .map(|t| (t, self.value_axis.label_style)),
            self.category_axis
                .title
                .as_deref()
                .map(|t| (t, self.category_axis.label_style)),
        );
        self.draw_legend(buf, plot);
    }

    fn draw_legend(&self, buf: &mut Buffer, plot: Rect) {
        let w = self
            .series_names
            .iter()
            .map(|n| text_width(n) + 2)
            .max()
            .unwrap_or(0);
        if w == 0 || w > plot.width {
            return;
        }
        for (i, name) in self.series_names.iter().enumerate() {
            if i as u16 >= plot.height {
                break;
            }
            let y = plot.y + i as u16;
            // legend sits on the top right; clear background to stay readable
            for x in plot.right() - w..plot.right() {
                buf.set_string(x, y, " ", Style::default());
            }
            buf.set_string(plot.right() - w, y, "■", Style::default().fg(self.color(i)));
            buf.set_string(
                plot.right() - w + 2,
                y,
                name,
                Style::default().fg(Color::Gray),
            );
        }
    }
}

impl Widget for &BarChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

impl Widget for BarChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn groups() -> Vec<BarGroup> {
        vec![
            BarGroup::new("Q1", [10.0, 20.0]),
            BarGroup::new("Q2", [30.0, 5.0]),
            BarGroup::new("Q3", [25.0, 15.0]),
        ]
    }

    fn dump(buf: &Buffer, area: Rect) -> String {
        (area.y..area.bottom())
            .flat_map(|y| (area.x..area.right()).map(move |x| (x, y)))
            .map(|p| buf[p].symbol().to_string())
            .collect()
    }

    #[test]
    fn bounds_grouped_vs_stacked() {
        let g = BarChart::new(groups());
        assert_eq!(g.bounds(), (0.0, 30.0));
        let s = BarChart::new(groups()).mode(BarMode::Stacked);
        assert_eq!(s.bounds().1, 40.0);
        assert_eq!(BarChart::new(groups()).max(100.0).bounds().1, 100.0);
        assert_eq!(BarChart::new(vec![]).bounds(), (0.0, 1.0));
    }

    #[test]
    fn negative_and_nan_values_clamped() {
        let c = BarChart::new(vec![BarGroup::new("a", [-5.0, f64::NAN, 3.0])]);
        assert_eq!(c.bounds().1, 3.0);
    }

    #[test]
    fn vertical_grouped_draws_colors() {
        let area = Rect::new(0, 0, 40, 14);
        let mut buf = Buffer::empty(area);
        BarChart::new(groups())
            .colors(vec![Color::Red, Color::Blue])
            .render(area, &mut buf);
        let fg: Vec<Color> = (0..40)
            .flat_map(|x| (0..14).map(move |y| (x, y)))
            .filter(|p| buf[*p].symbol() == "█")
            .map(|p| buf[p].fg)
            .collect();
        assert!(fg.contains(&Color::Red));
        assert!(fg.contains(&Color::Blue));
        assert!(dump(&buf, area).contains("Q1"));
    }

    #[test]
    fn vertical_stacked_uses_segment_colors() {
        let area = Rect::new(0, 0, 40, 14);
        let mut buf = Buffer::empty(area);
        BarChart::new(groups())
            .mode(BarMode::Stacked)
            .colors(vec![Color::Red, Color::Blue])
            .render(area, &mut buf);
        let fg: Vec<Color> = (0..40)
            .flat_map(|x| (0..14).map(move |y| (x, y)))
            .filter(|p| matches!(buf[*p].symbol(), "█" | "▄" | "▅" | "▆" | "▇"))
            .map(|p| buf[p].fg)
            .collect();
        assert!(fg.contains(&Color::Red) && fg.contains(&Color::Blue));
    }

    #[test]
    fn horizontal_draws_partial_blocks_and_values() {
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        BarChart::new(groups())
            .orientation(BarOrientation::Horizontal)
            .show_values(true)
            .render(area, &mut buf);
        let t = dump(&buf, area);
        assert!(t.contains('█'));
        assert!(t.contains("Q3"));
        assert!(t.contains("25"));
    }

    #[test]
    fn horizontal_stacked() {
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        BarChart::new(groups())
            .orientation(BarOrientation::Horizontal)
            .mode(BarMode::Stacked)
            .series_names(["a", "b"])
            .render(area, &mut buf);
        assert!(dump(&buf, area).contains('■'));
    }

    #[test]
    fn value_labels_vertical() {
        let area = Rect::new(0, 0, 40, 14);
        let mut buf = Buffer::empty(area);
        BarChart::new(groups())
            .bar_width(3)
            .show_values(true)
            .render(area, &mut buf);
        let t = dump(&buf, area);
        assert!(t.contains("10") || t.contains("20"));
    }

    #[test]
    fn many_groups_are_truncated_not_panicking() {
        let g: Vec<BarGroup> = (0..100)
            .map(|i| BarGroup::new(format!("{i}"), [i as f64]))
            .collect();
        let area = Rect::new(0, 0, 30, 10);
        let mut buf = Buffer::empty(area);
        BarChart::new(g.clone()).render(area, &mut buf);
        BarChart::new(g)
            .orientation(BarOrientation::Horizontal)
            .render(area, &mut buf);
    }

    #[test]
    fn tiny_and_empty_safe() {
        for (w, h) in [(0, 0), (2, 2), (8, 4), (10, 5)] {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            BarChart::new(groups()).render(area, &mut buf);
            BarChart::new(vec![]).render(area, &mut buf);
            BarChart::new(groups())
                .orientation(BarOrientation::Horizontal)
                .render(area, &mut buf);
        }
    }

    #[test]
    fn group_helper() {
        let g = BarGroup::new("x", [1.0, 2.0]);
        assert_eq!(g.label, "x");
        assert_eq!(g.values, vec![1.0, 2.0]);
    }
}
