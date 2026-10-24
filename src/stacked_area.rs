//! Stacked area chart: series stacked on top of each other, optionally
//! normalised to 100 %.

use crate::canvas::{Canvas, Marker};
use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::{darken, dim_text, format_tick, nice_bounds, nice_ticks, pick};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// One layer of a [`StackedAreaChart`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AreaLayer {
    /// Layer name (legend).
    pub name: String,
    /// Values at evenly spaced x positions; non-finite or negative values count as 0.
    pub values: Vec<f64>,
    /// Explicit color.
    pub color: Option<Color>,
}

impl AreaLayer {
    /// Create a layer.
    pub fn new(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            name: name.into(),
            values: values.into_iter().collect(),
            color: None,
        }
    }

    /// Explicit color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

/// Stacked area chart widget.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{AreaLayer, StackedAreaChart};
///
/// let chart = StackedAreaChart::new([AreaLayer::new("a", [1.0, 3.0, 2.0]), AreaLayer::new("b", [2.0, 1.0, 4.0])]);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct StackedAreaChart {
    layers: Vec<AreaLayer>,
    normalize: bool,
    legend: bool,
    outline: bool,
    hidden: Vec<usize>,
    hidden_legend: Vec<usize>,
    marker: Marker,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl StackedAreaChart {
    /// Create a chart.
    pub fn new(layers: impl IntoIterator<Item = AreaLayer>) -> Self {
        Self {
            layers: layers.into_iter().collect(),
            normalize: false,
            legend: true,
            outline: true,
            hidden: vec![],
            hidden_legend: vec![],
            marker: Marker::HalfBlock,
            theme: None,
            base: None,
            block: None,
        }
    }

    setter!(
        /// Scale every column to 100 % (default off).
        normalize: bool
    );
    setter!(
        /// Legend on the bottom row (default on, needs height 6+).
        legend: bool
    );
    setter!(
        /// Bright line on top of each layer (default on).
        outline: bool
    );
    setter!(
        /// Dot resolution (default half blocks, which fill solid; use braille for finer edges).
        marker: Marker
    );
    /// Hide layers by index (the stack re-flows; hidden layers stay in the
    /// legend so they can be toggled back, see [`Interaction`](crate::Interaction)).
    #[must_use]
    pub fn hidden(mut self, hidden: impl IntoIterator<Item = usize>) -> Self {
        self.hidden = hidden.into_iter().collect();
        self
    }
    theme_setter!(
        /// Apply a [`Theme`]; explicit layer colors win.
        theme
    );
    base_style_setter!(base);
    block_setter!(block, 'static);

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        if !self.hidden.is_empty() {
            let mut c = self.clone();
            for &h in &self.hidden {
                if let Some(l) = c.layers.get_mut(h) {
                    l.values.iter_mut().for_each(|v| *v = 0.0);
                }
            }
            c.hidden.clear();
            c.hidden_legend = self.hidden.clone();
            return c.render_into(area, buf);
        }
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        let n = self
            .layers
            .iter()
            .map(|l| l.values.len())
            .max()
            .unwrap_or(0);
        if n < 2 || inner.width < 8 || inner.height < 4 {
            return;
        }
        let clean = |v: f64| if v.is_finite() { v.max(0.0) } else { 0.0 };
        // cumulative tops per layer per column
        let mut tops: Vec<Vec<f64>> = Vec::with_capacity(self.layers.len());
        let mut run = vec![0.0; n];
        for l in &self.layers {
            for (i, r) in run.iter_mut().enumerate() {
                *r += clean(l.values.get(i).copied().unwrap_or(0.0));
            }
            tops.push(run.clone());
        }
        let totals = run;
        if self.normalize {
            for t in tops.iter_mut() {
                for (i, v) in t.iter_mut().enumerate() {
                    *v = if totals[i] > 0.0 {
                        *v / totals[i] * 100.0
                    } else {
                        0.0
                    };
                }
            }
        }
        let max = tops
            .last()
            .map_or(0.0, |t| t.iter().copied().fold(0.0, f64::max));
        if max <= 0.0 {
            return;
        }
        let (_, hi) = nice_bounds(0.0, max, 5);
        let (dim, text) = dim_text(&self.theme);
        let legend = self.legend && inner.height >= 6;
        let plot_h = inner.height - u16::from(legend);
        let ticks = nice_ticks(0.0, hi, (plot_h as usize / 3).max(2));
        let step = if ticks.len() >= 2 {
            ticks[1] - ticks[0]
        } else {
            1.0
        };
        let labels: Vec<String> = ticks
            .iter()
            .map(|t| {
                format!(
                    "{}{}",
                    format_tick(*t, step),
                    if self.normalize { "%" } else { "" }
                )
            })
            .collect();
        let gutter = (labels.iter().map(|l| l.chars().count()).max().unwrap_or(1) as u16 + 1)
            .min(inner.width / 2);
        let plot = Rect::new(inner.x + gutter, inner.y, inner.width - gutter, plot_h);
        if plot.width < 2 || plot.height < 2 {
            return;
        }
        for (t, l) in ticks.iter().zip(&labels) {
            let f = (t / hi).clamp(0.0, 1.0);
            let y = plot.bottom() - 1 - (f * (plot.height - 1) as f64).round() as u16;
            let w = l.chars().count() as u16;
            put_str(
                buf,
                inner.x + gutter.saturating_sub(w + 1),
                y,
                plot.x,
                l,
                dim,
            );
            for x in plot.x..plot.right() {
                buf[(x, y)].set_char('┈').set_style(dim);
            }
        }
        let mut cv = Canvas::new(plot.width, plot.height, self.marker);
        let (pw, ph) = (cv.px_w(), cv.px_h());
        let ypx = |v: f64| ((1.0 - v / hi) * (ph - 1) as f64).round() as i32;
        let at = |t: &[f64], px: i32| {
            let f = px as f64 / (pw - 1).max(1) as f64 * (n - 1) as f64;
            let i = (f.floor() as usize).min(n - 2);
            t[i] + (t[i + 1] - t[i]) * (f - i as f64)
        };
        // draw from the top layer down so lower layers overwrite shared edges
        for (li, l) in self.layers.iter().enumerate().rev() {
            let c = l.color.unwrap_or_else(|| pick(&self.theme, li));
            let fill = darken(c, 0.5);
            for px in 0..pw {
                let top = ypx(at(&tops[li], px));
                let bottom = if li == 0 {
                    ph - 1
                } else {
                    ypx(at(&tops[li - 1], px))
                };
                for py in top..=bottom {
                    cv.set(px, py, fill);
                }
                if self.outline {
                    cv.set(px, top, c);
                }
            }
        }
        cv.render(buf, plot);
        if crate::interact::recording() {
            let (rx, ry) = self.marker.resolution();
            let (rx, ry) = (i32::from(rx), i32::from(ry));
            for c in 0..plot.width {
                let px = (i32::from(c) * rx + rx / 2).min(pw - 1);
                let di = ((px as f64 / (pw - 1).max(1) as f64 * (n - 1) as f64).round() as usize)
                    .min(n - 1);
                for (li, l) in self.layers.iter().enumerate() {
                    if self.hidden_legend.contains(&li) {
                        continue;
                    }
                    let top = ypx(at(&tops[li], px)).max(0) / ry;
                    let bottom = if li == 0 {
                        i32::from(plot.height) - 1
                    } else {
                        ypx(at(&tops[li - 1], px)).max(0) / ry
                    };
                    if bottom < top {
                        continue;
                    }
                    let rect = Rect::new(
                        plot.x + c,
                        plot.y + top as u16,
                        1,
                        (bottom - top + 1) as u16,
                    )
                    .intersection(plot);
                    crate::interact::record_point(rect, li, di, || {
                        let v = l.values.get(di).copied().unwrap_or(0.0);
                        let mut lines = vec![crate::util::format_compact(clean(v))];
                        if totals[di] > 0.0 {
                            lines.push(format!("{:.0}% of total", clean(v) / totals[di] * 100.0));
                        }
                        (l.name.clone(), lines)
                    });
                }
            }
        }
        if legend {
            let mut x = plot.x;
            let y = inner.bottom() - 1;
            for (li, l) in self.layers.iter().enumerate() {
                let c = l.color.unwrap_or_else(|| pick(&self.theme, li));
                let w = l.name.chars().count() as u16 + 3;
                if x + w > inner.right() {
                    break;
                }
                let off = self.hidden_legend.contains(&li);
                put_str(
                    buf,
                    x,
                    y,
                    inner.right(),
                    if off { "□" } else { "■" },
                    Style::default().fg(c),
                );
                let name_style = if off {
                    text.add_modifier(
                        ratatui::style::Modifier::CROSSED_OUT | ratatui::style::Modifier::DIM,
                    )
                } else {
                    text
                };
                put_str(buf, x + 2, y, inner.right(), &l.name, name_style);
                crate::interact::record_legend(Rect::new(x, y, w, 1), li, &l.name);
                x += w + 1;
            }
        }
    }
}

widget_impls!(StackedAreaChart);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for StackedAreaChart {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(c: &StackedAreaChart, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    fn chart() -> StackedAreaChart {
        StackedAreaChart::new([
            AreaLayer::new("web", [1.0, 2.0, 3.0, 4.0]).color(Color::Red),
            AreaLayer::new("api", [2.0, 2.0, 2.0, 2.0]).color(Color::Blue),
        ])
    }

    #[test]
    fn draws_layers_and_legend() {
        let b = render(&chart(), 50, 14);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains("web") && t.contains("api"));
        assert!(b.content().iter().any(|c| c.fg == Color::Red));
        assert!(b.content().iter().any(|c| c.fg == Color::Blue));
    }

    #[test]
    fn stacking_puts_later_layers_above() {
        let b = render(&chart().legend(false), 50, 14);
        let first = |col: Color| {
            b.content()
                .iter()
                .position(|c| c.fg == col && c.symbol() != " ")
                .unwrap()
        };
        assert!(first(Color::Blue) < first(Color::Red) + 50 * 14);
        // topmost row with colour belongs to the last layer
        assert!(first(Color::Blue) <= first(Color::Red));
    }

    #[test]
    fn normalize_shows_percent_axis() {
        let b = render(&chart().normalize(true), 50, 14);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains('%'));
    }

    #[test]
    fn outline_toggle() {
        assert_ne!(
            render(&chart(), 50, 14),
            render(&chart().outline(false), 50, 14)
        );
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let c = StackedAreaChart::new([
            AreaLayer::new("n", [f64::NAN, f64::INFINITY, -3.0, 1e300]),
            AreaLayer::new("short", [1.0]),
        ]);
        for (w, h) in [(0, 0), (1, 1), (8, 4), (3, 40), (200, 60)] {
            render(&c, w, h);
            render(&chart().normalize(true), w, h);
        }
        render(&StackedAreaChart::new([]), 40, 10);
        render(
            &StackedAreaChart::new([AreaLayer::new("z", [0.0, 0.0])]),
            40,
            10,
        );
    }
}
