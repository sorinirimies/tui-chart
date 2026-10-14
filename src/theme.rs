//! Chart-specific theme glue on top of the shared [`tui_nuances::Theme`].
//!
//! The [`Theme`] type itself (built-ins, `from_base`, `next`/`prev`, serde)
//! lives in the shared `tui-nuances` crate and is re-exported here. This module
//! only adds what is chart-specific: [`ChartThemeExt`] (derived heatmap
//! gradient, crosshair/tooltip styles) and the [`begin`] render helper.
//!
//! Every widget has a `.theme(..)` builder (and implements
//! [`tui_nuances::Themed`]); explicit per-item colors (series colors, candle
//! colors, `.base_style(..)`, axis styles ...) always win over the theme,
//! regardless of call order.
//!
//! ```
//! use tui_chart::{LineChart, Series, Theme};
//!
//! let theme = Theme::dracula();
//! let chart = LineChart::new(vec![Series::new("a").values([1.0, 3.0, 2.0])]).theme(&theme);
//! # let _ = chart;
//! ```

use crate::heatmap::Gradient;
use ratatui::{buffer::Buffer, layout::Rect, style::Color, style::Style};
pub use tui_nuances::Theme;

/// Chart-specific helpers derived from a [`Theme`].
pub trait ChartThemeExt {
    /// Gradient from a dim tint of the background up to the accent and the
    /// second palette color (used by heatmap / treemap / sparkline).
    fn gradient(&self) -> Gradient;
    /// Crosshair guide line style.
    fn crosshair_style(&self) -> Style;
    /// Crosshair tooltip style (`tooltip_fg` on `tooltip_bg`).
    fn tooltip_style(&self) -> Style;
    /// Series color `i`, cycling through the palette.
    fn series_color(&self, i: usize) -> Color;
}

impl ChartThemeExt for Theme {
    fn gradient(&self) -> Gradient {
        let accent = to_rgb(self.accent);
        let low = match self.background {
            Some(bg) => mix(to_rgb(bg), accent, 0.2),
            None => mix((0, 0, 0), accent, 0.25),
        };
        let high = to_rgb(self.color(1));
        Gradient::Custom(vec![low, accent, high])
    }

    fn crosshair_style(&self) -> Style {
        Style::default().fg(self.crosshair)
    }

    fn tooltip_style(&self) -> Style {
        Style::default().fg(self.tooltip_fg).bg(self.tooltip_bg)
    }

    fn series_color(&self, i: usize) -> Color {
        self.color(i)
    }
}

/// Approximate RGB of any ratatui color.
pub(crate) fn to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::Red => (205, 49, 49),
        Color::Green => (13, 188, 121),
        Color::Yellow => (229, 229, 16),
        Color::Blue => (36, 114, 200),
        Color::Magenta => (188, 63, 188),
        Color::Cyan => (17, 168, 205),
        Color::Gray => (180, 180, 180),
        Color::DarkGray => (102, 102, 102),
        Color::LightRed => (241, 76, 76),
        Color::LightGreen => (35, 209, 139),
        Color::LightYellow => (245, 245, 67),
        Color::LightBlue => (59, 142, 234),
        Color::LightMagenta => (214, 112, 214),
        Color::LightCyan => (41, 184, 219),
        Color::White => (255, 255, 255),
        _ => (128, 128, 128),
    }
}

use crate::scale::mix_rgb as mix;

/// Render helper: fill background / base style, then the (themed) block.
/// Returns the inner area.
pub(crate) fn begin(
    block: &Option<ratatui::widgets::Block<'_>>,
    theme: &Option<Theme>,
    style: Option<Style>,
    area: Rect,
    buf: &mut Buffer,
) -> Rect {
    use ratatui::widgets::Widget;
    if let Some(t) = theme {
        t.fill(area, buf);
    }
    if let Some(s) = style {
        buf.set_style(area, s);
    }
    match block {
        Some(b) => {
            let inner = b.inner(area);
            let mut b = b.clone();
            if let Some(t) = theme {
                b = b
                    .style(Style::default().fg(t.border))
                    .title_style(Style::default().fg(t.title));
            }
            b.render(area, buf);
            inner
        }
        None => area,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_from_theme() {
        for t in Theme::all() {
            assert!(matches!(t.gradient(), Gradient::Custom(v) if v.len() == 3));
        }
    }

    #[test]
    fn ext_styles() {
        let t = Theme::nord();
        assert_eq!(t.crosshair_style().fg, Some(t.crosshair));
        assert_eq!(t.tooltip_style().bg, Some(t.tooltip_bg));
        assert_eq!(t.series_color(0), t.series_color(t.palette.len()));
    }

    #[test]
    fn fill_paints_background() {
        let mut b = Buffer::empty(Rect::new(0, 0, 3, 2));
        Theme::dracula().fill(Rect::new(0, 0, 3, 2), &mut b);
        assert_eq!(b[(1, 1)].bg, Theme::dracula().background.unwrap());
    }
}
