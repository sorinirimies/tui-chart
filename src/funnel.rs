//! Funnel: stages narrowing from the largest value, with conversion percentages.

use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::format_compact;
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
#[derive(Debug, Clone, Default)]
pub struct Funnel {
    stages: Vec<FunnelStage>,
    theme: Option<Theme>,
    base: Option<Style>,
    show_percent: bool,
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
            block: None,
        }
    }

    setter!(
        /// Show conversion percentage vs. the first stage (default on).
        show_percent: bool
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
        let max = self
            .stages
            .iter()
            .map(|s| {
                if s.value.is_finite() {
                    s.value.max(0.0)
                } else {
                    0.0
                }
            })
            .fold(0.0, f64::max);
        let conv = self.conversions();
        for (i, st) in self.stages.iter().enumerate() {
            let y0 = inner.y as u32 + i as u32 * rows_each as u32;
            if y0 >= inner.bottom() as u32 {
                break;
            }
            let h = (rows_each as u32).min(inner.bottom() as u32 - y0) as u16;
            let v = if st.value.is_finite() {
                st.value.max(0.0)
            } else {
                0.0
            };
            let frac = if max > 0.0 { v / max } else { 0.0 };
            let bw = ((frac * inner.width as f64).round() as u16).clamp(1, inner.width);
            let x0 = inner.x + (inner.width - bw) / 2;
            let color = st.color.unwrap_or_else(|| {
                self.theme
                    .as_ref()
                    .map_or(PALETTE[i % PALETTE.len()], |t| t.color(i))
            });
            let bar_h = if rows_each > 2 { h - 1 } else { h };
            for y in y0 as u16..y0 as u16 + bar_h {
                for x in x0..x0 + bw {
                    buf[(x, y)].set_char('█').set_fg(color);
                }
            }
            let mut label = format!("{} {}", st.label, format_compact(v));
            if self.show_percent {
                label.push_str(&format!(" ({:.0}%)", conv[i] * 100.0));
            }
            let lw = label.chars().count() as u16;
            let lx = inner.x + inner.width.saturating_sub(lw) / 2;
            // On the bar (contrasting fg) when it fits; otherwise in the gap row
            // below the bar, or on the bar when there is no gap.
            let on_bar = lw <= bw;
            let (ly, style) = if on_bar {
                (
                    y0 as u16 + bar_h.saturating_sub(1) / 2,
                    Style::default().fg(Color::Black).bg(color),
                )
            } else if bar_h < h {
                (y0 as u16 + bar_h, text)
            } else {
                (y0 as u16 + bar_h.saturating_sub(1) / 2, text)
            };
            put_str(buf, lx, ly, inner.right(), &label, style);
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
}
