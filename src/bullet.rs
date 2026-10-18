//! Bullet chart: a value bar against a target marker over qualitative ranges.

use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::{darken, format_compact};
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
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{Bullet, BulletItem};
///
/// let chart = Bullet::new([BulletItem::new("Revenue", 270.0).target(300.0)]);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
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
            |t| {
                let f = t.foreground;
                [darken(f, 0.68), darken(f, 0.8), darken(f, 0.9)]
            },
        );
        let col = |v: f64, max: f64| -> u16 {
            ((fin(v) / max).clamp(0.0, 1.0) * (bw - 1) as f64).round() as u16
        };
        // Row height 3 → 2 → 1 (with a blank spacer row when it fits).
        let n = self.items.len();
        let avail = inner.height as usize;
        let (rh, gap) = [(3usize, 1usize), (2, 1), (1, 1), (1, 0)]
            .into_iter()
            .find(|(h, g)| n * h + n.saturating_sub(1) * g <= avail)
            .unwrap_or((1, 0));
        let total = n * rh + n.saturating_sub(1) * gap;
        let top = inner.y as usize + avail.saturating_sub(total) / 2;
        let good = Color::Rgb(80, 200, 120);
        let warn = Color::Rgb(240, 180, 60);
        for (i, it) in self.items.iter().enumerate() {
            let y0 = top + i * (rh + gap);
            if y0 >= inner.bottom() as usize {
                break;
            }
            let rh = rh.min(inner.bottom() as usize - y0) as u16;
            let y0 = y0 as u16;
            let ymid = y0 + rh / 2;
            put_str(buf, inner.x, ymid, bx, &it.label, text);
            let max = self.scale_max(it);
            // qualitative ranges: lightest = worst; no ranges → one neutral band
            let nb = it.ranges.len();
            let mut shade_at = vec![bands[2]; bw as usize];
            let mut from = 0u16;
            for (k, r) in it.ranges.iter().enumerate() {
                let to = col(*r, max) + 1;
                let shade = bands[(k * 3 / nb.max(1)).min(2)];
                for x in from..to.min(bw) {
                    shade_at[x as usize] = shade;
                }
                from = from.max(to);
            }
            for r in 0..rh {
                for x in 0..bw {
                    buf[(bx + x, y0 + r)]
                        .set_char(' ')
                        .set_bg(shade_at[x as usize]);
                }
            }
            // value bar: eighth-block precision on the middle row (lower half if 1 row)
            let frac = (fin(it.value) / max).clamp(0.0, 1.0);
            let bar_y = y0 + if rh == 1 { 0 } else { rh / 2 };
            if rh == 1 {
                let end = col(it.value, max) + u16::from(fin(it.value) > 0.0);
                for x in 0..end.min(bw) {
                    buf[(bx + x, bar_y)].set_char('▄').set_fg(bar);
                }
            } else {
                let eighths = (frac * bw as f64 * 8.0).round() as u32;
                for x in 0..bw {
                    let rem = eighths.saturating_sub(x as u32 * 8);
                    let cell = &mut buf[(bx + x, bar_y)];
                    if rem >= 8 {
                        cell.set_char('█').set_fg(bar);
                    } else if rem > 0 {
                        cell.set_char(['▏', '▎', '▍', '▌', '▋', '▊', '▉'][rem as usize - 1])
                            .set_fg(bar);
                    }
                }
            }
            if let Some(t) = it.target {
                let x = col(t, max).min(bw - 1);
                for r in 0..rh {
                    buf[(bx + x, y0 + r)]
                        .set_char(if rh == 1 { '│' } else { '┃' })
                        .set_fg(marker);
                }
            }
            let mut vstyle = text;
            let mut note = String::new();
            if let Some(t) = it.target.filter(|t| fin(*t) > 0.0) {
                let pct = fin(it.value) / fin(t) * 100.0;
                vstyle = vstyle.fg(if pct >= 100.0 { good } else { warn });
                note = format!(" {pct:.0}%");
            }
            let vx = bx + bw + 1;
            put_str(
                buf,
                vx,
                ymid,
                inner.right(),
                &format_compact(fin(it.value)),
                vstyle,
            );
            if !note.is_empty() && rh >= 2 && ymid + 1 < y0 + rh {
                put_str(
                    buf,
                    vx,
                    ymid + 1,
                    inner.right(),
                    note.trim_start(),
                    Style::default().fg(darken(marker, 0.35)),
                );
            }
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
        assert!(all.contains('█') && all.contains('┃') && all.contains("Rev"));
        let one = render(&b, 40, 1);
        let all: String = one.content().iter().map(|c| c.symbol()).collect();
        assert!(all.contains('▄') && all.contains('│'));
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

    #[test]
    fn custom_color_and_max() {
        let b = Bullet::new([BulletItem::new("a", 50.0)])
            .max(200.0)
            .color(Color::Magenta);
        let buf = render(&b, 40, 2);
        assert!(buf.content().iter().any(|c| c.fg == Color::Magenta));
        // a larger axis max shrinks the bar
        let full = |m: f64| {
            render(&Bullet::new([BulletItem::new("a", 50.0)]).max(m), 40, 2)
                .content()
                .iter()
                .filter(|c| c.symbol() == "█")
                .count()
        };
        assert!(full(100.0) > full(400.0));
    }

    #[test]
    fn many_items_and_row_heights() {
        let items = (0..6).map(|i| BulletItem::new(format!("m{i}"), i as f64 * 10.0));
        let b = Bullet::new(items);
        for h in [1, 2, 3, 6, 18, 40] {
            render(&b, 50, h);
        }
    }
}
