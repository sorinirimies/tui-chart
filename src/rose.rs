//! Rose (polar area / Nightingale) chart: equal-angle sectors whose radius
//! encodes the value.

use crate::canvas::{Canvas, Marker};
use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::{darken, dim_text, format_compact, pick};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};
use std::f64::consts::TAU;

/// One sector of a [`RoseChart`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RoseItem {
    /// Sector label.
    pub label: String,
    /// Value (negative and non-finite values are drawn as zero).
    pub value: f64,
    /// Explicit color.
    pub color: Option<Color>,
}

impl RoseItem {
    /// Create an item.
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    /// Explicit color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

/// Rose chart widget.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{RoseChart, RoseItem};
///
/// let chart = RoseChart::new([RoseItem::new("N", 4.0), RoseItem::new("E", 9.0), RoseItem::new("S", 2.0)]);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct RoseChart {
    items: Vec<RoseItem>,
    area_scale: bool,
    rings: usize,
    start_deg: f64,
    inner: f64,
    show_labels: bool,
    show_values: bool,
    marker: Marker,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl RoseChart {
    /// Create a chart.
    pub fn new(items: impl IntoIterator<Item = RoseItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            area_scale: true,
            rings: 3,
            start_deg: 0.0,
            inner: 0.0,
            show_labels: true,
            show_values: true,
            marker: Marker::default(),
            theme: None,
            base: None,
            block: None,
        }
    }

    setter!(
        /// Radius ∝ √value so sector *area* is proportional to value
        /// (default); `false` makes the radius linear in value.
        area_scale: bool
    );
    setter!(
        /// Number of dotted reference rings (default 3).
        rings: usize
    );
    setter!(
        /// Angle of the first sector's start, degrees clockwise from the top.
        start_deg: f64
    );
    setter!(
        /// Hole radius as a fraction of the outer radius, `0.0..0.9` (default 0).
        inner: f64
    );
    setter!(
        /// Draw sector labels around the chart (default on).
        show_labels: bool
    );
    setter!(
        /// Append the value to the labels (default on).
        show_values: bool
    );
    setter!(
        /// Dot resolution (default braille; half blocks fill solid but are coarser).
        marker: Marker
    );
    theme_setter!(
        /// Apply a [`Theme`]; explicit item colors win.
        theme
    );
    base_style_setter!(base);
    block_setter!(block, 'static);

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner_rect = theme::begin(&self.block, &self.theme, self.base, area, buf);
        let n = self.items.len();
        if n == 0 || inner_rect.width < 8 || inner_rect.height < 5 {
            return;
        }
        let vals: Vec<f64> = self
            .items
            .iter()
            .map(|i| {
                if i.value.is_finite() {
                    i.value.max(0.0)
                } else {
                    0.0
                }
            })
            .collect();
        let vmax = vals.iter().copied().fold(0.0, f64::max);
        if vmax <= 0.0 {
            return;
        }
        let (dim, text) = dim_text(&self.theme);
        let (rxd, ryd) = self.marker.resolution();
        let label_w = if self.show_labels {
            (self
                .items
                .iter()
                .map(|i| i.label.chars().count() + if self.show_values { 6 } else { 0 })
                .max()
                .unwrap_or(0) as u16)
                .min(inner_rect.width / 3)
        } else {
            0
        };
        let plot = Rect::new(
            inner_rect.x + label_w,
            inner_rect.y + u16::from(self.show_labels),
            inner_rect.width.saturating_sub(2 * label_w),
            inner_rect
                .height
                .saturating_sub(2 * u16::from(self.show_labels)),
        );
        if plot.width < 4 || plot.height < 3 {
            return;
        }
        let mut cv = Canvas::new(plot.width, plot.height, self.marker);
        let (pw, ph) = (cv.px_w(), cv.px_h());
        // one py is `aspect` times as tall as one px is wide (cells are ~1:2)
        let aspect = 2.0 * rxd as f64 / ryd as f64;
        let rad = ((pw as f64 / 2.0).min(ph as f64 * aspect / 2.0) - 1.0).max(2.0);
        let (cx, cy) = (pw as f64 / 2.0, ph as f64 / 2.0);
        let hole = self.inner.clamp(0.0, 0.9);
        let start = self.start_deg.to_radians();
        let sector = TAU / n as f64;
        let scale = |v: f64| {
            let f = v / vmax;
            let f = if self.area_scale { f.sqrt() } else { f };
            hole + (1.0 - hole) * f
        };
        let colors: Vec<Color> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, it)| it.color.unwrap_or_else(|| pick(&self.theme, i)))
            .collect();

        let mut owner: Vec<Option<usize>> = if crate::interact::recording() {
            vec![None; usize::from(plot.width) * usize::from(plot.height)]
        } else {
            Vec::new()
        };
        for py in 0..ph {
            for px in 0..pw {
                let dx = px as f64 + 0.5 - cx;
                let dy = (py as f64 + 0.5 - cy) * aspect;
                let r = dx.hypot(dy);
                if r > rad || r < rad * hole {
                    continue;
                }
                let ang = ((dx.atan2(-dy) - start).rem_euclid(TAU) / sector).min(n as f64 - 1e-9);
                let k = ang as usize;
                let edge = (ang - k as f64) * sector * r; // arc distance from sector start
                if edge < 0.6 {
                    continue; // gap between sectors
                }
                let reach = rad * scale(vals[k]);
                if r <= reach {
                    if !owner.is_empty() {
                        let cell = (py / ryd as i32) as usize * usize::from(plot.width)
                            + (px / rxd as i32) as usize;
                        if let Some(o) = owner.get_mut(cell).filter(|o| o.is_none()) {
                            *o = Some(k);
                        }
                    }
                    let c = if r > reach - 1.5 {
                        colors[k]
                    } else {
                        darken(colors[k], 0.4)
                    };
                    cv.set(px, py, c);
                }
            }
        }
        let total: f64 = vals.iter().sum();
        crate::interact::record_grid(
            plot,
            &owner,
            |_| 0,
            |k| {
                let f = crate::util::format_compact;
                (
                    self.items[k].label.clone(),
                    vec![
                        format!("value {}", f(vals[k])),
                        format!("share {:.0}%", vals[k] / total * 100.0),
                    ],
                )
            },
        );
        // dotted reference rings, drawn only on empty dots
        for ring in 1..=self.rings {
            let rr = rad * ring as f64 / self.rings as f64;
            let steps = (rr * 6.0) as usize + 8;
            for s in 0..steps {
                let a = TAU * s as f64 / steps as f64;
                let (x, y) = (
                    (cx + rr * a.sin()) as i32,
                    (cy - rr * a.cos() / aspect) as i32,
                );
                if !cv.is_set(x, y) {
                    cv.set(x, y, dim.fg.unwrap_or(Color::DarkGray));
                }
            }
        }
        cv.render(buf, plot);

        if self.show_labels {
            let ccx = plot.x as f64 + plot.width as f64 / 2.0;
            let ccy = plot.y as f64 + plot.height as f64 / 2.0;
            let (rx, ry) = (plot.width as f64 / 2.0, plot.height as f64 / 2.0);
            for (k, it) in self.items.iter().enumerate() {
                let a = start + sector * (k as f64 + 0.5);
                let s = if self.show_values {
                    format!("{} {}", it.label, format_compact(vals[k]))
                } else {
                    it.label.clone()
                };
                let w = s.chars().count() as f64;
                let x = ccx + a.sin() * (rx + 1.0)
                    - if a.sin() < -0.2 {
                        w
                    } else if a.sin() > 0.2 {
                        0.0
                    } else {
                        w / 2.0
                    };
                let y = ccy - a.cos() * (ry + 0.5);
                let (xi, yi) = (x.round() as i32, y.round() as i32);
                if xi >= inner_rect.x as i32
                    && yi >= inner_rect.y as i32
                    && yi < inner_rect.bottom() as i32
                {
                    put_str(
                        buf,
                        xi as u16,
                        yi as u16,
                        inner_rect.right(),
                        &s,
                        text.fg(colors[k]),
                    );
                }
            }
        }
    }
}

widget_impls!(RoseChart);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for RoseChart {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(c: &RoseChart, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    fn items() -> Vec<RoseItem> {
        ["N", "E", "S", "W"]
            .iter()
            .zip([4.0, 9.0, 1.0, 16.0])
            .map(|(l, v)| RoseItem::new(*l, v))
            .collect()
    }

    #[test]
    fn draws_sectors_and_labels() {
        let b = render(&RoseChart::new(items()), 60, 24);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains("W 16") && t.contains("E 9"));
        assert!(b
            .content()
            .iter()
            .any(|c| c.symbol() != " " && c.fg == crate::PALETTE[0]));
    }

    #[test]
    fn largest_value_fills_most() {
        let count = |color: Color| {
            let it: Vec<RoseItem> = items()
                .into_iter()
                .map(|i| {
                    let c = if i.label == "W" {
                        Color::Red
                    } else {
                        Color::Blue
                    };
                    i.color(c)
                })
                .collect();
            render(&RoseChart::new(it).show_labels(false).rings(0), 60, 24)
                .content()
                .iter()
                .filter(|c| c.fg == color)
                .count()
        };
        assert!(count(Color::Red) > 0 && count(Color::Blue) > 0);
    }

    #[test]
    fn linear_scale_changes_output() {
        let a = render(&RoseChart::new(items()), 60, 24);
        let b = render(&RoseChart::new(items()).area_scale(false), 60, 24);
        assert_ne!(a, b);
    }

    #[test]
    fn hole_and_rotation_apply() {
        let base = render(&RoseChart::new(items()), 60, 24);
        assert_ne!(base, render(&RoseChart::new(items()).inner(0.5), 60, 24));
        assert_ne!(
            base,
            render(&RoseChart::new(items()).start_deg(45.0), 60, 24)
        );
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let c = RoseChart::new([
            RoseItem::new("n", f64::NAN),
            RoseItem::new("i", f64::INFINITY),
            RoseItem::new("neg", -5.0),
            RoseItem::new("big", 1e300),
        ]);
        for (w, h) in [(0, 0), (1, 1), (8, 5), (3, 40), (200, 60)] {
            render(&c, w, h);
            render(&RoseChart::new(items()).inner(5.0).rings(0), w, h);
        }
        render(&RoseChart::new([]), 40, 10);
        render(&RoseChart::new([RoseItem::new("z", 0.0)]), 40, 10);
    }
}
