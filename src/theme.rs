//! Color themes shared by every chart widget.
//!
//! A [`Theme`] bundles the colors a chart needs: background/foreground, axis,
//! grid, border and title colors, a series palette, bull/bear/neutral colors
//! (candles, depth, waterfall, renko, box plots), an accent color and crosshair
//! tooltip colors. Every widget has a `.theme(..)` builder; explicit per-item
//! colors (series colors, candle colors, ...) always win over the theme.
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

/// A named set of colors applied to charts via `.theme(..)`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Theme {
    /// Display name (also used by [`Theme::next`] / [`Theme::prev`]).
    pub name: String,
    /// Background fill; `None` keeps the terminal background.
    pub background: Option<Color>,
    /// Default text color.
    pub foreground: Color,
    /// De-emphasised text (tick labels, legends, secondary text).
    pub muted: Color,
    /// Axis line color.
    pub axis: Color,
    /// Grid line color.
    pub grid: Color,
    /// Block border color.
    pub border: Color,
    /// Title color.
    pub title: Color,
    /// Series palette (at least 8 colors); series `i` uses `palette[i % len]`.
    pub palette: Vec<Color>,
    /// "Up" color (bullish candles, bids, increases).
    pub bull: Color,
    /// "Down" color (bearish candles, asks, decreases).
    pub bear: Color,
    /// Neutral / total color.
    pub neutral: Color,
    /// Accent color (single-series charts, gradients, highlights).
    pub accent: Color,
    /// Crosshair guide line color.
    pub crosshair: Color,
    /// Crosshair tooltip text color.
    pub tooltip_fg: Color,
    /// Crosshair tooltip background color.
    pub tooltip_bg: Color,
}

const fn rgb(hex: u32) -> Color {
    Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

impl Default for Theme {
    /// Terminal-native theme: matches the colors used when no theme is set.
    fn default() -> Self {
        Self {
            name: "default".into(),
            background: None,
            foreground: Color::White,
            muted: Color::Gray,
            axis: Color::DarkGray,
            grid: Color::DarkGray,
            border: Color::Gray,
            title: Color::White,
            palette: crate::PALETTE.to_vec(),
            bull: Color::Green,
            bear: Color::Red,
            neutral: Color::Blue,
            accent: Color::Cyan,
            crosshair: Color::Gray,
            tooltip_fg: Color::White,
            tooltip_bg: Color::DarkGray,
        }
    }
}

impl From<&Theme> for Theme {
    fn from(t: &Theme) -> Self {
        t.clone()
    }
}

#[allow(clippy::too_many_arguments)]
fn make(
    name: &str,
    bg: u32,
    fg: u32,
    muted: u32,
    grid: u32,
    accent: u32,
    bull: u32,
    bear: u32,
    palette: [u32; 8],
) -> Theme {
    Theme {
        name: name.into(),
        background: Some(rgb(bg)),
        foreground: rgb(fg),
        muted: rgb(muted),
        axis: rgb(muted),
        grid: rgb(grid),
        border: rgb(muted),
        title: rgb(fg),
        palette: palette.iter().map(|&c| rgb(c)).collect(),
        bull: rgb(bull),
        bear: rgb(bear),
        neutral: rgb(muted),
        accent: rgb(accent),
        crosshair: rgb(muted),
        tooltip_fg: rgb(fg),
        tooltip_bg: rgb(grid),
    }
}

impl Theme {
    /// Catppuccin Mocha (dark).
    pub fn catppuccin_mocha() -> Self {
        make(
            "catppuccin-mocha",
            0x1e1e2e,
            0xcdd6f4,
            0x9399b2,
            0x45475a,
            0x89b4fa,
            0xa6e3a1,
            0xf38ba8,
            [
                0x89b4fa, 0xf9e2af, 0xcba6f7, 0xa6e3a1, 0xf38ba8, 0x94e2d5, 0xfab387, 0xf5c2e7,
            ],
        )
    }

    /// Catppuccin Latte (light).
    pub fn catppuccin_latte() -> Self {
        make(
            "catppuccin-latte",
            0xeff1f5,
            0x4c4f69,
            0x7c7f93,
            0xccd0da,
            0x1e66f5,
            0x40a02b,
            0xd20f39,
            [
                0x1e66f5, 0xdf8e1d, 0x8839ef, 0x40a02b, 0xd20f39, 0x179299, 0xfe640b, 0xea76cb,
            ],
        )
    }

    /// Dracula.
    pub fn dracula() -> Self {
        make(
            "dracula",
            0x282a36,
            0xf8f8f2,
            0x6272a4,
            0x44475a,
            0xbd93f9,
            0x50fa7b,
            0xff5555,
            [
                0x8be9fd, 0xf1fa8c, 0xff79c6, 0x50fa7b, 0xff5555, 0xbd93f9, 0xffb86c, 0xf8f8f2,
            ],
        )
    }

    /// Nord.
    pub fn nord() -> Self {
        make(
            "nord",
            0x2e3440,
            0xd8dee9,
            0x7b88a1,
            0x434c5e,
            0x88c0d0,
            0xa3be8c,
            0xbf616a,
            [
                0x88c0d0, 0xebcb8b, 0xb48ead, 0xa3be8c, 0xbf616a, 0x81a1c1, 0xd08770, 0xe5e9f0,
            ],
        )
    }

    /// Gruvbox (dark).
    pub fn gruvbox() -> Self {
        make(
            "gruvbox",
            0x282828,
            0xebdbb2,
            0xa89984,
            0x3c3836,
            0xfabd2f,
            0xb8bb26,
            0xfb4934,
            [
                0x83a598, 0xfabd2f, 0xd3869b, 0xb8bb26, 0xfb4934, 0x8ec07c, 0xfe8019, 0xebdbb2,
            ],
        )
    }

    /// Tokyo Night.
    pub fn tokyo_night() -> Self {
        make(
            "tokyo-night",
            0x1a1b26,
            0xc0caf5,
            0x787c99,
            0x292e42,
            0x7aa2f7,
            0x9ece6a,
            0xf7768e,
            [
                0x7aa2f7, 0xe0af68, 0xbb9af7, 0x9ece6a, 0xf7768e, 0x7dcfff, 0xff9e64, 0xc0caf5,
            ],
        )
    }

    /// Solarized Dark.
    pub fn solarized_dark() -> Self {
        make(
            "solarized-dark",
            0x002b36,
            0x93a1a1,
            0x657b83,
            0x073642,
            0x268bd2,
            0x859900,
            0xdc322f,
            [
                0x268bd2, 0xb58900, 0xd33682, 0x859900, 0xdc322f, 0x2aa198, 0xcb4b16, 0x6c71c4,
            ],
        )
    }

    /// Porsche: black/white with the signature red accent (`#D5001C`).
    pub fn porsche() -> Self {
        let mut t = make(
            "porsche",
            0x000000,
            0xffffff,
            0x9b9ba0,
            0x2b2b2e,
            0xd5001c,
            0xffffff,
            0xd5001c,
            [
                0xd5001c, 0xffffff, 0x9b9ba0, 0xff5a6e, 0x6b6b70, 0xe3e3e6, 0x8c0014, 0xc4c4c8,
            ],
        );
        t.neutral = rgb(0x9b9ba0);
        t
    }

    /// Colorblind-safe theme (Okabe–Ito palette; blue/orange instead of green/red).
    pub fn colorblind() -> Self {
        Theme {
            name: "colorblind".into(),
            background: None,
            foreground: Color::White,
            muted: Color::Gray,
            axis: Color::DarkGray,
            grid: Color::DarkGray,
            border: Color::Gray,
            title: Color::White,
            palette: [
                0x0072b2, 0xe69f00, 0x009e73, 0xf0e442, 0x56b4e9, 0xd55e00, 0xcc79a7, 0xbbbbbb,
            ]
            .iter()
            .map(|&c| rgb(c))
            .collect(),
            bull: rgb(0x0072b2),
            bear: rgb(0xe69f00),
            neutral: rgb(0xbbbbbb),
            accent: rgb(0x56b4e9),
            crosshair: Color::Gray,
            tooltip_fg: Color::White,
            tooltip_bg: Color::DarkGray,
        }
    }

    /// All built-in themes, in cycling order.
    pub fn all() -> Vec<Theme> {
        vec![
            Theme::default(),
            Theme::catppuccin_mocha(),
            Theme::catppuccin_latte(),
            Theme::dracula(),
            Theme::nord(),
            Theme::gruvbox(),
            Theme::tokyo_night(),
            Theme::solarized_dark(),
            Theme::porsche(),
            Theme::colorblind(),
        ]
    }

    fn position(&self) -> usize {
        Theme::all()
            .iter()
            .position(|t| t.name == self.name)
            .unwrap_or(0)
    }

    /// Next built-in theme (wraps around; unknown names start from the first).
    pub fn next(&self) -> Theme {
        let all = Theme::all();
        all[(self.position() + 1) % all.len()].clone()
    }

    /// Previous built-in theme (wraps around).
    pub fn prev(&self) -> Theme {
        let all = Theme::all();
        all[(self.position() + all.len() - 1) % all.len()].clone()
    }

    /// Set the display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set (or clear) the background fill.
    pub fn background(mut self, color: Option<Color>) -> Self {
        self.background = color;
        self
    }

    /// Set the default text color.
    pub fn foreground(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }

    /// Set the muted text color.
    pub fn muted(mut self, color: Color) -> Self {
        self.muted = color;
        self
    }

    /// Set the axis line color.
    pub fn axis(mut self, color: Color) -> Self {
        self.axis = color;
        self
    }

    /// Set the grid line color.
    pub fn grid(mut self, color: Color) -> Self {
        self.grid = color;
        self
    }

    /// Set the border color.
    pub fn border(mut self, color: Color) -> Self {
        self.border = color;
        self
    }

    /// Set the title color.
    pub fn title(mut self, color: Color) -> Self {
        self.title = color;
        self
    }

    /// Set the series palette (an empty palette is ignored).
    pub fn palette(mut self, palette: Vec<Color>) -> Self {
        if !palette.is_empty() {
            self.palette = palette;
        }
        self
    }

    /// Set the bull (up) color.
    pub fn bull(mut self, color: Color) -> Self {
        self.bull = color;
        self
    }

    /// Set the bear (down) color.
    pub fn bear(mut self, color: Color) -> Self {
        self.bear = color;
        self
    }

    /// Set the neutral color.
    pub fn neutral(mut self, color: Color) -> Self {
        self.neutral = color;
        self
    }

    /// Set the accent color.
    pub fn accent(mut self, color: Color) -> Self {
        self.accent = color;
        self
    }

    /// Set the crosshair guide color.
    pub fn crosshair(mut self, color: Color) -> Self {
        self.crosshair = color;
        self
    }

    /// Set the tooltip foreground/background colors.
    pub fn tooltip(mut self, fg: Color, bg: Color) -> Self {
        self.tooltip_fg = fg;
        self.tooltip_bg = bg;
        self
    }

    /// Palette color for series `i` (cycles).
    pub fn color(&self, i: usize) -> Color {
        if self.palette.is_empty() {
            self.accent
        } else {
            self.palette[i % self.palette.len()]
        }
    }

    /// Base style: foreground (+ background when set).
    pub fn base_style(&self) -> Style {
        let s = Style::default().fg(self.foreground);
        match self.background {
            Some(bg) => s.bg(bg),
            None => s,
        }
    }

    /// Axis line style.
    pub fn axis_style(&self) -> Style {
        Style::default().fg(self.axis)
    }

    /// Tick label / legend style.
    pub fn label_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Grid style.
    pub fn grid_style(&self) -> Style {
        Style::default().fg(self.grid)
    }

    /// Border style.
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Title style.
    pub fn title_style(&self) -> Style {
        Style::default().fg(self.title)
    }

    /// Gradient from a dim tint of the background up to the accent and the
    /// second palette color (used by heatmap / treemap / sparkline).
    pub fn gradient(&self) -> Gradient {
        let accent = to_rgb(self.accent);
        let low = match self.background {
            Some(bg) => mix(to_rgb(bg), accent, 0.2),
            None => mix((0, 0, 0), accent, 0.25),
        };
        let high = to_rgb(self.color(1));
        Gradient::Custom(vec![low, accent, high])
    }

    /// Fill `area` with the background color (no-op when `background` is `None`).
    pub fn fill(&self, area: Rect, buf: &mut Buffer) {
        if let Some(bg) = self.background {
            buf.set_style(area, Style::default().fg(self.foreground).bg(bg));
        }
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

fn mix(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let l = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * t).round() as u8;
    (l(a.0, b.0), l(a.1, b.1), l(a.2, b.2))
}

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
                b = b.style(t.border_style()).title_style(t.title_style());
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
    fn all_have_palette() {
        for t in Theme::all() {
            assert!(t.palette.len() >= 8, "{}", t.name);
        }
    }

    #[test]
    fn cycling_wraps() {
        let mut t = Theme::default();
        let n = Theme::all().len();
        for _ in 0..n {
            t = t.next();
        }
        assert_eq!(t.name, "default");
        assert_eq!(
            Theme::default().prev().name,
            Theme::all().last().unwrap().name
        );
    }

    #[test]
    fn color_cycles() {
        let t = Theme::nord();
        assert_eq!(t.color(0), t.color(t.palette.len()));
    }

    #[test]
    fn fill_paints_background() {
        let mut b = Buffer::empty(Rect::new(0, 0, 3, 2));
        Theme::dracula().fill(Rect::new(0, 0, 3, 2), &mut b);
        assert_eq!(b[(1, 1)].bg, Theme::dracula().background.unwrap());
        let mut b = Buffer::empty(Rect::new(0, 0, 3, 2));
        Theme::default().fill(Rect::new(0, 0, 3, 2), &mut b);
        assert_eq!(b[(1, 1)].bg, Color::Reset);
    }

    #[test]
    fn porsche_accent() {
        assert_eq!(Theme::porsche().accent, Color::Rgb(0xd5, 0x00, 0x1c));
    }
}
