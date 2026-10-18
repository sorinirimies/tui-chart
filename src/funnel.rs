//! Funnel: stages narrowing from the largest value, with conversion percentages.

use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::{darken, format_compact};
use crate::PALETTE;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// One funnel stage.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunnelStage {
    /// Stage label.
    pub label: String,
    /// Stage value (non-finite / negative values count as 0).
    pub value: f64,
    /// Explicit color.
    pub color: Option<Color>,
}

impl FunnelStage {
    /// Create a stage.
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    /// Explicit stage color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

/// Funnel chart widget: centered bars whose width follows the value.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{Funnel, FunnelStage};
///
/// let chart = Funnel::new([FunnelStage::new("Visits", 1000.0), FunnelStage::new("Signups", 240.0)]);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Funnel {
    stages: Vec<FunnelStage>,
    theme: Option<Theme>,
    base: Option<Style>,
    show_percent: bool,
    show_dropoff: bool,
    taper: bool,
    block: Option<Block<'static>>,
}

impl Funnel {
    /// Create a funnel from stages (first stage is the 100 % reference).
    pub fn new(stages: impl IntoIterator<Item = FunnelStage>) -> Self {
        Self {
            stages: stages.into_iter().collect(),
            theme: None,
            base: None,
            show_percent: true,
            show_dropoff: true,
            taper: true,
            block: None,
        }
    }

    setter!(
        /// Show conversion percentage vs. the first stage (default on).
        show_percent: bool
    );

    setter!(
        /// Show the drop-off vs. the previous stage (default on).
        show_dropoff: bool
    );

    setter!(
        /// Connect stages into one tapering funnel (default on); off draws separate bars.
        taper: bool
    );

    theme_setter!(
        /// Apply a [`Theme`]; explicit stage colors win.
        theme
    );

    base_style_setter!(base);

    block_setter!(block, 'static);

    /// Conversion of each stage relative to the first (0..=1; 0 if the first is 0).
    pub fn conversions(&self) -> Vec<f64> {
        let v = |s: &FunnelStage| {
            if s.value.is_finite() {
                s.value.max(0.0)
            } else {
                0.0
            }
        };
        let first = self.stages.first().map_or(0.0, v);
        self.stages
            .iter()
            .map(|s| if first > 0.0 { v(s) / first } else { 0.0 })
            .collect()
    }

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 6 || inner.height == 0 || self.stages.is_empty() {
            return;
        }
        let n = self.stages.len() as u16;
        let rows_each = (inner.height / n).max(1);
        let text = self
            .theme
            .as_ref()
            .map_or(Style::default(), |t| t.label_style());
        let dim = Style::default().fg(darken(
            self.theme.as_ref().map_or(Color::Gray, |t| t.foreground),
            0.4,
        ));
        let vals: Vec<f64> = self
            .stages
            .iter()
            .map(|s| {
                if s.value.is_finite() {
                    s.value.max(0.0)
                } else {
                    0.0
                }
            })
            .collect();
        let max = vals.iter().copied().fold(0.0, f64::max);
        let conv = self.conversions();

        // Columns: [labels] [funnel] [value + percent + drop-off]. Side columns
        // are dropped on narrow areas, falling back to centered labels.
        let label_w = (self
            .stages
            .iter()
            .map(|s| s.label.chars().count())
            .max()
            .unwrap_or(0) as u16
            + 2)
        .min(inner.width / 4);
        let stat_w: u16 = if self.show_percent { 16 } else { 8 };
        let side = inner.width >= label_w + stat_w + 16;
        let (fx, fw) = if side {
            (inner.x + label_w, inner.width - label_w - stat_w)
        } else {
            (inner.x, inner.width)
        };
        let center = fx as f64 + fw as f64 / 2.0;

        for (i, st) in self.stages.iter().enumerate() {
            let y0 = inner.y as u32 + i as u32 * rows_each as u32;
            if y0 >= inner.bottom() as u32 {
                break;
            }
            let h = (rows_each as u32).min(inner.bottom() as u32 - y0) as u16;
            let color = st.color.unwrap_or_else(|| {
                self.theme
                    .as_ref()
                    .map_or(PALETTE[i % PALETTE.len()], |t| t.color(i))
            });
            let frac = |v: f64| if max > 0.0 { v / max } else { 0.0 };
            let top_w = (frac(vals[i]) * fw as f64).max(1.0);
            // bottom edge meets the next stage's top; the last stage narrows to a neck
            let bot_w = if !self.taper {
                top_w
            } else if let Some(nv) = vals.get(i + 1) {
                (frac(*nv) * fw as f64).max(1.0)
            } else {
                (top_w * 0.7).max(1.0)
            };
            let bar_h = if self.taper || rows_each <= 2 {
                h
            } else {
                h - 1
            };
            for r in 0..bar_h {
                let t = (r as f64 + 0.5) / bar_h as f64;
                let w = top_w + (bot_w - top_w) * t;
                let (l, rt) = (center - w / 2.0, center + w / 2.0);
                let y = y0 as u16 + r;
                let first = (l.floor() as i64).max(fx as i64) as u16;
                let last = ((rt.ceil() as i64).min((fx + fw) as i64) - 1).max(first as i64) as u16;
                for x in first..=last.min(fx + fw - 1) {
                    let cover_l = (x as f64).max(l);
                    let cover_r = (x as f64 + 1.0).min(rt);
                    let cover = (cover_r - cover_l).max(0.0);
                    let cell = &mut buf[(x, y)];
                    if cover >= 0.75 {
                        cell.set_char('█').set_fg(color);
                    } else if cover >= 0.25 {
                        // partial edge cell: half block on the side that is covered
                        let left_side = (x as f64) < l + 0.5 && l > x as f64;
                        cell.set_char(if left_side { '▐' } else { '▌' })
                            .set_fg(color);
                    }
                }
            }

            // value / percent / drop-off (side) or centered label (narrow)
            let mid = y0 as u16 + bar_h.saturating_sub(1) / 2;
            if side {
                put_str(buf, inner.x, mid, fx, &st.label, text);
                let mut stat = format_compact(vals[i]);
                if self.show_percent {
                    stat.push_str(&format!("  {:>3.0}%", conv[i] * 100.0));
                }
                put_str(buf, fx + fw + 1, mid, inner.right(), &stat, text);
                if self.show_percent && self.show_dropoff && i > 0 && vals[i - 1] > 0.0 {
                    let drop = (1.0 - vals[i] / vals[i - 1]) * 100.0;
                    if h > 1 {
                        let dy = y0 as u16;
                        if dy != mid {
                            put_str(
                                buf,
                                fx + fw + 1,
                                dy,
                                inner.right(),
                                &format!("▼ {drop:.0}%"),
                                dim,
                            );
                        }
                    }
                }
            } else {
                let mut label = format!("{} {}", st.label, format_compact(vals[i]));
                if self.show_percent {
                    label.push_str(&format!(" ({:.0}%)", conv[i] * 100.0));
                }
                let lw = label.chars().count() as u16;
                let lx = inner.x + inner.width.saturating_sub(lw) / 2;
                let on_bar = (lw as f64) <= top_w.min(bot_w);
                let style = if on_bar {
                    Style::default().fg(Color::Black).bg(color)
                } else {
                    text
                };
                put_str(buf, lx, mid, inner.right(), &label, style);
            }
        }
    }
}

widget_impls!(Funnel);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for Funnel {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn funnel() -> Funnel {
        Funnel::new([
            FunnelStage::new("Visits", 1000.0),
            FunnelStage::new("Signups", 400.0),
            FunnelStage::new("Paid", 100.0),
        ])
    }

    #[test]
    fn conversions_relative_to_first() {
        assert_eq!(funnel().conversions(), vec![1.0, 0.4, 0.1]);
        assert_eq!(
            Funnel::new([FunnelStage::new("a", 0.0)]).conversions(),
            vec![0.0]
        );
    }

    #[test]
    fn renders_labels_and_percent() {
        let area = Rect::new(0, 0, 50, 9);
        let mut b = Buffer::empty(area);
        funnel().render(area, &mut b);
        let all: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(all.contains("Visits") && all.contains("40%") && all.contains("10%"));
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let mut b = Buffer::empty(Rect::new(0, 0, 3, 1));
        funnel().render(Rect::new(0, 0, 3, 1), &mut b);
        funnel().render(Rect::new(0, 0, 0, 0), &mut b);
        Funnel::new([FunnelStage::new("x", f64::NAN)]).render(Rect::new(0, 0, 3, 1), &mut b);
        let mut b = Buffer::empty(Rect::new(0, 0, 30, 2));
        funnel().render(Rect::new(0, 0, 30, 2), &mut b);
    }

    fn text(b: &Buffer) -> String {
        b.content().iter().map(|c| c.symbol()).collect()
    }

    fn draw(f: Funnel, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        f.render(a, &mut b);
        b
    }

    #[test]
    fn dropoff_toggle() {
        let on = text(&draw(funnel(), 60, 9));
        let off = text(&draw(funnel().show_dropoff(false), 60, 9));
        assert!(on.contains('▼') && !off.contains('▼'));
    }

    #[test]
    fn percent_toggle() {
        let off = text(&draw(funnel().show_percent(false), 60, 9));
        assert!(!off.contains("40%"));
    }

    #[test]
    fn taper_changes_output_and_custom_color_applies() {
        let f = || funnel();
        assert_ne!(
            text(&draw(f().taper(false), 60, 9)),
            text(&draw(f().taper(true), 60, 9))
        );
        let b = draw(
            Funnel::new([FunnelStage::new("a", 5.0).color(Color::Red)]),
            40,
            4,
        );
        assert!(b
            .content()
            .iter()
            .any(|c| c.fg == Color::Red || c.bg == Color::Red));
    }

    #[test]
    fn narrow_and_tiny_areas_do_not_panic() {
        for (w, h) in [(0, 0), (1, 1), (5, 2), (12, 3), (200, 40)] {
            draw(funnel(), w, h);
            draw(funnel().taper(false).show_dropoff(false), w, h);
        }
    }
}
