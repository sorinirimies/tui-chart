//! Accessibility helpers: render any chart without relying on colour.
//!
//! [`Monochrome`] wraps a widget, renders it normally and then post-processes
//! the cells it touched:
//!
//! * solid colour fills (a coloured *background* on a blank cell, as drawn by
//!   heatmaps, treemaps, calendars, bars) become shade glyphs `░▒▓█` chosen by
//!   luminance, so intensity is still readable;
//! * coloured foreground glyphs keep their shape, lose their colour, and dark
//!   (low luminance) ones get the `DIM` modifier so contrast between series
//!   survives;
//! * all colours are reset, so the terminal's own foreground/background apply.
//!
//! [`Monochrome::auto`] only does this when the `NO_COLOR` environment
//! variable is set (<https://no-color.org>), so one code path serves both.
//!
//! ```
//! use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
//! use tui_chart::{Heatmap, Monochrome};
//!
//! let area = Rect::new(0, 0, 20, 6);
//! let mut buf = Buffer::empty(area);
//! Monochrome::new(Heatmap::new(vec![vec![0.0, 0.5, 1.0]]).cell_width(4)).render(area, &mut buf);
//! assert!(buf.content().iter().any(|c| matches!(c.symbol(), "░" | "▒" | "▓" | "█")));
//! ```

use crate::util::to_rgb;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    widgets::Widget,
};

/// `true` when the `NO_COLOR` environment variable is set to a non-empty value.
pub fn no_color_requested() -> bool {
    std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty())
}

fn luminance(c: Color) -> f64 {
    let (r, g, b) = to_rgb(c);
    (0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64) / 255.0
}

/// Strip colour from `area` of `buf`, replacing solid fills by shade glyphs.
pub fn strip_color(buf: &mut Buffer, area: Rect) {
    const SHADES: [&str; 5] = [" ", "░", "▒", "▓", "█"];
    let area = area.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            let blank = cell.symbol().trim().is_empty();
            let (fg, bg) = (cell.fg, cell.bg);
            if blank && bg != Color::Reset {
                let l = luminance(bg);
                let i = (l * (SHADES.len() - 1) as f64).round() as usize;
                cell.set_symbol(SHADES[i.min(SHADES.len() - 1)]);
            } else if !blank && fg != Color::Reset && luminance(fg) < 0.35 {
                cell.modifier.insert(Modifier::DIM);
            }
            cell.set_fg(Color::Reset);
            cell.set_bg(Color::Reset);
        }
    }
}

/// Renders the wrapped widget, then removes its colours (see the module docs).
#[derive(Debug, Clone)]
pub struct Monochrome<W> {
    inner: W,
    only_if_no_color: bool,
}

impl<W> Monochrome<W> {
    /// Always render without colour.
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            only_if_no_color: false,
        }
    }

    /// Render without colour only when `NO_COLOR` is set.
    pub fn auto(inner: W) -> Self {
        Self {
            inner,
            only_if_no_color: true,
        }
    }
}

impl<W: Widget> Widget for Monochrome<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let strip = !self.only_if_no_color || no_color_requested();
        self.inner.render(area, buf);
        if strip {
            strip_color(buf, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BarChart, BarGroup, Heatmap, LineChart, Series};

    fn has_color(b: &Buffer) -> bool {
        b.content()
            .iter()
            .any(|c| c.fg != Color::Reset || c.bg != Color::Reset)
    }

    #[test]
    fn removes_all_colors() {
        let area = Rect::new(0, 0, 40, 12);
        let mut plain = Buffer::empty(area);
        LineChart::new(vec![
            Series::new("a").data((0..20).map(|i| (i as f64, i as f64)))
        ])
        .render(area, &mut plain);
        assert!(has_color(&plain));
        let mut mono = Buffer::empty(area);
        Monochrome::new(LineChart::new(vec![
            Series::new("a").data((0..20).map(|i| (i as f64, i as f64)))
        ]))
        .render(area, &mut mono);
        assert!(!has_color(&mono));
        // glyph shapes survive
        let glyphs = |b: &Buffer| {
            b.content()
                .iter()
                .map(|c| c.symbol().to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(glyphs(&plain), glyphs(&mono));
    }

    #[test]
    fn fills_become_shades_by_intensity() {
        let area = Rect::new(0, 0, 24, 3);
        let mut b = Buffer::empty(area);
        Monochrome::new(
            Heatmap::new(vec![vec![0.0, 0.25, 0.5, 0.75, 1.0]])
                .cell_width(4)
                .legend(false),
        )
        .render(area, &mut b);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(!has_color(&b));
        assert!(t.contains('▓') || t.contains('█'));
        assert!(t.contains('░') || t.contains('▒') || t.contains(' '));
    }

    #[test]
    fn auto_follows_no_color_env() {
        let area = Rect::new(0, 0, 30, 8);
        let mk = || BarChart::new(vec![BarGroup::new("a", [3.0, 5.0])]);
        // without NO_COLOR: untouched (assumes the test env does not set it)
        if !no_color_requested() {
            let mut b = Buffer::empty(area);
            Monochrome::auto(mk()).render(area, &mut b);
            assert!(has_color(&b));
        }
        let mut b = Buffer::empty(area);
        Monochrome::new(mk()).render(area, &mut b);
        assert!(!has_color(&b));
    }

    #[test]
    fn handles_out_of_bounds_areas() {
        let mut b = Buffer::empty(Rect::new(0, 0, 5, 5));
        strip_color(&mut b, Rect::new(3, 3, 50, 50));
        strip_color(&mut b, Rect::new(0, 0, 0, 0));
    }
}
