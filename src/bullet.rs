//! Bullet chart: a value bar against a target marker over qualitative ranges.

use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::format_compact;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// One bullet row.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BulletItem {
    /// Row label.
    pub label: String,
    /// Measured value.
    pub value: f64,
    /// Target marker position.
    pub target: Option<f64>,
    /// Ascending upper bounds of the qualitative ranges (e.g. poor/ok/good).
    pub ranges: Vec<f64>,
}

impl BulletItem {
    /// Create an item with no target or ranges.
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            target: None,
            ranges: vec![],
        }
    }

    /// Target marker.
    pub fn target(mut self, t: f64) -> Self {
        self.target = Some(t);
        self
    }

    /// Ascending range bounds.
    pub fn ranges(mut self, r: impl IntoIterator<Item = f64>) -> Self {
        self.ranges = r.into_iter().collect();
        self
    }
}

/// Bullet chart widget (one row per item, shared scale `0..max`).
#[derive(Debug, Clone, Default)]
pub struct Bullet {
    items: Vec<BulletItem>,
    max: Option<f64>,
    color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

fn fin(v: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

impl Bullet {
    /// Create a bullet chart.
    pub fn new(items: impl IntoIterator<Item = BulletItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            ..Self::default()
        }
    }

    /// Fixed scale maximum shared by all rows (default: each row is scaled to
    /// its own largest value/target/range, so mixed units work).
    pub fn max(mut self, m: f64) -> Self {
        self.max = Some(m);
        self
    }

    /// Value bar color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }

    theme_setter!(
        /// Apply a [`Theme`]; explicit colors win.
        theme
    );

    base_style_setter!(base);

    block_setter!(block, 'static);

    fn scale_max(&self, it: &BulletItem) -> f64 {
        let m = self.max.map(fin).unwrap_or_else(|| {
            it.ranges
                .iter()
                .copied()
                .chain([it.value])
                .chain(it.target)
                .map(fin)
                .fold(0.0, f64::max)
        });
        if m > 0.0 {
            m
        } else {
            1.0
        }
    }

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 8 || inner.height == 0 || self.items.is_empty() {
            return;
        }
        let lw = (self
            .items
            .iter()
            .map(|i| i.label.chars().count())
            .max()
            .unwrap_or(0) as u16)
            .min(inner.width / 3);
        let vw = (self
            .items
            .iter()
            .map(|i| format_compact(fin(i.value)).chars().count())
            .max()
            .unwrap_or(0) as u16
            + 1)
        .min(inner.width / 4);
        let bx = inner.x + lw + 1;
        let bw = inner.width.saturating_sub(lw + 1 + vw);
        if bw < 2 {
            return;
        }
        let th = self.theme.as_ref();
        let text = th.map_or(Style::default(), |t| t.label_style());
        let bar = self
            .color
            .unwrap_or_else(|| th.map_or(Color::Cyan, |t| t.accent));
        let marker = th.map_or(Color::White, |t| t.foreground);
        let bands: [Color; 3] = th.map_or(
            [
                Color::Rgb(70, 70, 70),
                Color::Rgb(52, 52, 52),
                Color::Rgb(36, 36, 36),
            ],
            |t| [t.muted, t.border, t.grid],
        );
        let col = |v: f64, max: f64| -> u16 {
            ((fin(v) / max).clamp(0.0, 1.0) * (bw - 1) as f64).round() as u16
        };
        let stride = if inner.height as usize >= self.items.len() * 2 {
            2
        } else {
            1
        };
        for (i, it) in self.items.iter().enumerate() {
            let y = inner.y as u32 + (i * stride) as u32;
            if y >= inner.bottom() as u32 {
                break;
            }
            let y = y as u16;
            put_str(buf, inner.x, y, bx, &it.label, text);
            let max = self.scale_max(it);
            // qualitative ranges: lightest = worst; no ranges → one neutral band
            let nb = it.ranges.len();
            let mut from = 0u16;
            for (k, r) in it.ranges.iter().enumerate() {
                let to = col(*r, max) + 1;
                let shade = bands[(k * 3 / nb.max(1)).min(2)];
                for x in from..to.min(bw) {
                    buf[(bx + x, y)].set_char(' ').set_bg(shade);
                }
                from = from.max(to);
            }
            for x in from..bw {
                buf[(bx + x, y)].set_char(' ').set_bg(bands[2]);
            }
            // value bar (thin, so the ranges stay visible above it)
            let end = col(it.value, max) + u16::from(fin(it.value) > 0.0);
            for x in 0..end.min(bw) {
                buf[(bx + x, y)].set_char('▄').set_fg(bar);
            }
            if let Some(t) = it.target {
                let x = col(t, max).min(bw - 1);
                buf[(bx + x, y)].set_char('│').set_fg(marker);
            }
            put_str(
                buf,
                bx + bw + 1,
                y,
                inner.right(),
                &format_compact(fin(it.value)),
                text,
            );
        }
    }
}

widget_impls!(Bullet);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for Bullet {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(b: &Bullet, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(a);
        b.render(a, &mut buf);
        buf
    }

    #[test]
    fn draws_value_target_and_ranges() {
        let b = Bullet::new([BulletItem::new("Rev", 70.0)
            .target(85.0)
            .ranges([50.0, 80.0, 100.0])]);
        let buf = render(&b, 40, 2);
        let all: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(all.contains('▄') && all.contains('│') && all.contains("Rev"));
        assert!(buf.content().iter().any(|c| c.bg != Color::Reset));
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let b = Bullet::new([BulletItem::new("x", f64::NAN).target(f64::INFINITY)]);
        render(&b, 40, 1);
        render(&b, 3, 1);
        render(&b, 0, 0);
        render(&Bullet::new([]), 20, 3);
        render(
            &Bullet::new([BulletItem::new("a", -5.0).ranges([f64::NAN])]).max(0.0),
            20,
            3,
        );
    }
}
