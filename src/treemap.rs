//! Treemap: squarified treemap of weighted (optionally nested) items.

use crate::theme::{self, ChartThemeExt, Theme};
use crate::util::format_compact;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

const PALETTE: [(u8, u8, u8); 8] = [
    (66, 133, 244),
    (52, 168, 83),
    (251, 188, 5),
    (234, 67, 53),
    (171, 71, 188),
    (0, 172, 193),
    (255, 112, 67),
    (141, 110, 99),
];

/// One treemap item. A leaf uses `value`; an item with children is sized by
/// the sum of its children.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TreemapItem {
    /// Label drawn inside the cell.
    pub label: String,
    /// Weight (leaf items).
    pub value: f64,
    /// Explicit color (overrides gradient/palette).
    pub color: Option<Color>,
    /// Nested items.
    pub children: Vec<TreemapItem>,
}

impl TreemapItem {
    /// Create a leaf item.
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
            children: vec![],
        }
    }

    opt_setter!(
        /// Set an explicit color.
        color: Color
    );

    /// Set nested children (turns this item into a group).
    pub fn children(mut self, c: impl IntoIterator<Item = TreemapItem>) -> Self {
        self.children = c.into_iter().collect();
        self
    }

    /// Effective weight (sum of children when present). Invalid → 0.
    pub fn weight(&self) -> f64 {
        let v = if self.children.is_empty() {
            self.value
        } else {
            self.children.iter().map(|c| c.weight()).sum()
        };
        if v.is_finite() && v > 0.0 {
            v
        } else {
            0.0
        }
    }
}

/// Squarified treemap widget.
#[derive(Debug, Clone)]
pub struct Treemap {
    items: Vec<TreemapItem>,
    gradient: Option<(Color, Color)>,
    border: bool,
    border_color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    show_values: bool,
    block: Option<Block<'static>>,
}

impl Treemap {
    /// Create from items.
    pub fn new(items: impl IntoIterator<Item = TreemapItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            gradient: None,
            border: false,
            border_color: None,
            theme: None,
            base: None,
            show_values: true,
            block: None,
        }
    }

    /// Color items without explicit color by value, interpolating `low`→`high`
    /// (use `Color::Rgb` values for smooth results).
    pub fn gradient(mut self, low: Color, high: Color) -> Self {
        self.gradient = Some((low, high));
        self
    }

    setter!(
        /// Draw a border around each cell.
        border: bool
    );

    opt_setter!(
        /// Border color.
        border_color: Color
    );

    theme_setter!(
        /// Apply a [`Theme`]: items without a color are colored by value along the
        /// theme gradient; cell borders use the theme background. Explicit item
        /// colors, [`Treemap::gradient`] and [`Treemap::border_color`] win.
        theme
    );

    base_style_setter!(base);

    setter!(
        /// Print the value under the label when there is room.
        show_values: bool
    );

    block_setter!(block, 'static);

    /// Compute squarified cell rects for `weights` inside `area`, in the
    /// input order. Zero-weight items get empty rects.
    pub fn layout(weights: &[f64], area: Rect) -> Vec<Rect> {
        let mut out = vec![Rect::new(area.x, area.y, 0, 0); weights.len()];
        let mut order: Vec<usize> = (0..weights.len())
            .filter(|&i| weights[i].is_finite() && weights[i] > 0.0)
            .collect();
        order.sort_by(|&a, &b| weights[b].total_cmp(&weights[a]));
        let total: f64 = order.iter().map(|&i| weights[i]).sum();
        if order.is_empty() || area.width == 0 || area.height == 0 || total <= 0.0 {
            return out;
        }
        // Virtual space: cells are ~2x taller than wide.
        let (ax, ay) = (area.x as f64, area.y as f64 * 2.0);
        let (mut x, mut y) = (ax, ay);
        let (mut w, mut h) = (area.width as f64, area.height as f64 * 2.0);
        let scale = w * h / total;
        let mut i = 0;
        let mut rects: Vec<(usize, f64, f64, f64, f64)> = vec![];
        while i < order.len() {
            let short = w.min(h);
            let areas: Vec<f64> = order[i..].iter().map(|&k| weights[k] * scale).collect();
            let worst = |n: usize| -> f64 {
                let s: f64 = areas[..n].iter().sum();
                let mx = areas[..n].iter().cloned().fold(f64::MIN, f64::max);
                let mn = areas[..n].iter().cloned().fold(f64::MAX, f64::min);
                let s2 = short * short;
                ((s2 * mx) / (s * s)).max((s * s) / (s2 * mn))
            };
            let mut n = 1;
            while n < areas.len() && worst(n + 1) <= worst(n) {
                n += 1;
            }
            let s: f64 = areas[..n].iter().sum();
            if w >= h {
                let cw = if h > 0.0 { s / h } else { 0.0 };
                let mut cy = y;
                for k in 0..n {
                    let ch = if cw > 0.0 { areas[k] / cw } else { 0.0 };
                    rects.push((order[i + k], x, cy, cw, ch));
                    cy += ch;
                }
                x += cw;
                w -= cw;
            } else {
                let rh = if w > 0.0 { s / w } else { 0.0 };
                let mut cx = x;
                for k in 0..n {
                    let cw = if rh > 0.0 { areas[k] / rh } else { 0.0 };
                    rects.push((order[i + k], cx, y, cw, rh));
                    cx += cw;
                }
                y += rh;
                h -= rh;
            }
            i += n;
        }
        let (r_right, r_bottom) = (area.right() as f64, area.bottom() as f64);
        for (k, rx, ry, rw, rh) in rects {
            let x0 = rx.round().clamp(ax, r_right);
            let x1 = (rx + rw).round().clamp(ax, r_right);
            let y0 = (ry / 2.0).round().clamp(area.y as f64, r_bottom);
            let y1 = ((ry + rh) / 2.0).round().clamp(area.y as f64, r_bottom);
            out[k] = Rect::new(x0 as u16, y0 as u16, (x1 - x0) as u16, (y1 - y0) as u16);
        }
        out
    }

    fn leaf_range(items: &[TreemapItem], acc: &mut (f64, f64)) {
        for it in items {
            if it.children.is_empty() {
                let w = it.weight();
                if w > 0.0 {
                    acc.0 = acc.0.min(w);
                    acc.1 = acc.1.max(w);
                }
            } else {
                Self::leaf_range(&it.children, acc);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw(&self, items: &[TreemapItem], area: Rect, buf: &mut Buffer, range: (f64, f64)) {
        let weights: Vec<f64> = items.iter().map(|i| i.weight()).collect();
        let rects = Self::layout(&weights, area);
        for (idx, (it, r)) in items.iter().zip(rects).enumerate() {
            if r.width == 0 || r.height == 0 {
                continue;
            }
            let color = it
                .color
                .unwrap_or_else(|| self.auto_color(idx, weights[idx], range));
            let bg = Style::default().bg(color).fg(text_color(color));
            for y in r.y..r.bottom() {
                for x in r.x..r.right() {
                    buf[(x, y)].set_symbol(" ").set_style(bg);
                }
            }
            if self.border && r.width >= 2 && r.height >= 2 {
                self.draw_border(buf, r, color);
            }
            let inset = if self.border && r.width >= 2 && r.height >= 2 {
                1
            } else {
                0
            };
            let text = Rect::new(
                r.x + inset,
                r.y + inset,
                r.width.saturating_sub(2 * inset),
                r.height.saturating_sub(2 * inset),
            );
            if text.width == 0 || text.height == 0 {
                continue;
            }
            put_text(
                buf,
                text,
                0,
                &it.label,
                bg.add_modifier(ratatui::style::Modifier::BOLD),
            );
            if it.children.is_empty() {
                if self.show_values && text.height >= 2 {
                    put_text(buf, text, 1, &format_compact(weights[idx]), bg);
                }
            } else if text.height >= 2 {
                let sub = Rect::new(text.x, text.y + 1, text.width, text.height - 1);
                self.draw(&it.children, sub, buf, range);
            }
        }
    }

    fn draw_border(&self, buf: &mut Buffer, r: Rect, bg: Color) {
        let st = Style::default()
            .bg(bg)
            .fg(self.border_color.unwrap_or_else(|| {
                self.theme
                    .as_ref()
                    .and_then(|t| t.background)
                    .unwrap_or(Color::Black)
            }));
        let (x1, y1) = (r.right() - 1, r.bottom() - 1);
        for x in r.x..=x1 {
            buf[(x, r.y)].set_symbol("─").set_style(st);
            buf[(x, y1)].set_symbol("─").set_style(st);
        }
        for y in r.y..=y1 {
            buf[(r.x, y)].set_symbol("│").set_style(st);
            buf[(x1, y)].set_symbol("│").set_style(st);
        }
        buf[(r.x, r.y)].set_symbol("┌");
        buf[(x1, r.y)].set_symbol("┐");
        buf[(r.x, y1)].set_symbol("└");
        buf[(x1, y1)].set_symbol("┘");
    }

    fn auto_color(&self, idx: usize, weight: f64, range: (f64, f64)) -> Color {
        if let Some((lo, hi)) = self.gradient {
            let t = if range.1 > range.0 {
                ((weight - range.0) / (range.1 - range.0)).clamp(0.0, 1.0)
            } else {
                0.5
            };
            return lerp_color(lo, hi, t);
        }
        if let Some(t) = &self.theme {
            let f = if range.1 > range.0 {
                ((weight - range.0) / (range.1 - range.0)).clamp(0.0, 1.0)
            } else {
                0.5
            };
            return t.gradient().color_at(f);
        }
        let p = PALETTE[idx % PALETTE.len()];
        Color::Rgb(p.0, p.1, p.2)
    }
}

fn rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::Red => (205, 49, 49),
        Color::Green => (13, 188, 121),
        Color::Yellow => (229, 229, 16),
        Color::Blue => (36, 114, 200),
        Color::Magenta => (188, 63, 188),
        Color::Cyan => (17, 168, 205),
        Color::Gray => (229, 229, 229),
        Color::DarkGray => (102, 102, 102),
        Color::White => (255, 255, 255),
        _ => (200, 200, 200),
    }
}

fn lerp_color(a: Color, b: Color, t: f64) -> Color {
    let (a, b) = (rgb(a), rgb(b));
    let (r, g, bl) = crate::scale::mix_rgb(a, b, t);
    Color::Rgb(r, g, bl)
}

fn text_color(bg: Color) -> Color {
    let (r, g, b) = rgb(bg);
    if 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64 > 140.0 {
        Color::Black
    } else {
        Color::White
    }
}

fn put_text(buf: &mut Buffer, area: Rect, row: u16, s: &str, style: Style) {
    if row >= area.height {
        return;
    }
    for (i, ch) in s.chars().take(area.width as usize).enumerate() {
        buf[(area.x + i as u16, area.y + row)]
            .set_symbol(&ch.to_string())
            .set_style(style);
    }
}

impl Treemap {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        let mut range = (f64::INFINITY, f64::NEG_INFINITY);
        Treemap::leaf_range(&self.items, &mut range);
        self.draw(&self.items, inner, buf, range);
    }
}

widget_impls!(Treemap);

impl tui_nuances::Themed for Treemap {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    #[test]
    fn layout_tiles_area() {
        let area = Rect::new(2, 1, 40, 12);
        let rects = Treemap::layout(&[6.0, 6.0, 4.0, 3.0, 2.0, 2.0, 1.0], area);
        let covered: u32 = rects.iter().map(|r| r.width as u32 * r.height as u32).sum();
        assert_eq!(covered, 40 * 12);
        for r in &rects {
            assert!(area.contains((r.x, r.y).into()));
            assert!(r.right() <= area.right() && r.bottom() <= area.bottom());
        }
    }

    #[test]
    fn area_proportional() {
        let rects = Treemap::layout(&[3.0, 1.0], Rect::new(0, 0, 40, 10));
        let a = rects[0].width as u32 * rects[0].height as u32;
        let b = rects[1].width as u32 * rects[1].height as u32;
        assert_eq!(a, 300);
        assert_eq!(b, 100);
    }

    #[test]
    fn invalid_weights_empty() {
        let r = Treemap::layout(&[0.0, f64::NAN, -1.0], Rect::new(0, 0, 10, 10));
        assert!(r.iter().all(|r| r.width == 0 || r.height == 0));
    }

    #[test]
    fn nested_weight() {
        let g = TreemapItem::new("g", 0.0)
            .children([TreemapItem::new("a", 2.0), TreemapItem::new("b", 3.0)]);
        assert_eq!(g.weight(), 5.0);
    }

    #[test]
    fn renders_labels_and_colors() {
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        Treemap::new([
            TreemapItem::new("alpha", 5.0).color(Color::Red),
            TreemapItem::new("beta", 3.0),
            TreemapItem::new("gamma", 2.0),
        ])
        .render(area, &mut buf);
        let t: String = (0..12)
            .flat_map(|y| (0..40).map(move |x| (x, y)))
            .map(|p| buf[p].symbol().to_string())
            .collect();
        assert!(t.contains("alpha") && t.contains("beta"));
        assert!((0..40).all(|x| (0..12).all(|y| buf[(x, y)].bg != Color::Reset)));
    }

    #[test]
    fn theme_gradient_and_override() {
        let t = Theme::dracula();
        let area = Rect::new(0, 0, 40, 12);
        let items = || {
            [
                TreemapItem::new("alpha", 5.0).color(Color::Red),
                TreemapItem::new("beta", 3.0),
                TreemapItem::new("gamma", 1.0),
            ]
        };
        let mut buf = Buffer::empty(area);
        Treemap::new(items()).theme(&t).render(area, &mut buf);
        let g = t.gradient();
        let has = |b: &Buffer, c: Color| b.content().iter().any(|x| x.bg == c);
        assert!(has(&buf, Color::Red));
        assert!(has(&buf, g.color_at(1.0)) || has(&buf, g.color_at(0.5)));
        assert!(has(&buf, g.color_at(0.0)));
        let mut buf = Buffer::empty(area);
        Treemap::new(items())
            .theme(&t)
            .gradient(Color::Rgb(1, 2, 3), Color::Rgb(1, 2, 3))
            .render(area, &mut buf);
        assert!(has(&buf, Color::Rgb(1, 2, 3)) && !has(&buf, g.color_at(0.0)));
    }

    #[test]
    fn border_and_nested() {
        let area = Rect::new(0, 0, 40, 14);
        let mut buf = Buffer::empty(area);
        Treemap::new([
            TreemapItem::new("grp", 0.0)
                .children([TreemapItem::new("x", 4.0), TreemapItem::new("y", 2.0)]),
            TreemapItem::new("solo", 3.0),
        ])
        .border(true)
        .gradient(Color::Rgb(0, 0, 80), Color::Rgb(0, 200, 0))
        .block(Block::bordered())
        .render(area, &mut buf);
        let t: String = (0..14)
            .flat_map(|y| (0..40).map(move |x| (x, y)))
            .map(|p| buf[p].symbol().to_string())
            .collect();
        assert!(t.contains("grp") && t.contains("solo") && t.contains('┌'));
    }

    #[test]
    fn robust() {
        for (w, h) in [(0, 0), (1, 1), (2, 3), (5, 2)] {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            Treemap::new([TreemapItem::new("a", 1.0), TreemapItem::new("b", f64::NAN)])
                .border(true)
                .block(Block::bordered())
                .render(area, &mut buf);
        }
        let area = Rect::new(0, 0, 10, 5);
        Treemap::new([]).render(area, &mut Buffer::empty(area));
    }
}
