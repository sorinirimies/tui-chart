//! Gauge: linear / bullet / arc gauge with threshold bands and target marker.

use crate::canvas::{Canvas, Marker};
use crate::theme::{self, Theme};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// Direction of a linear gauge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GaugeOrientation {
    /// Fills left to right.
    #[default]
    Horizontal,
    /// Fills bottom to top.
    Vertical,
}

/// Visual form of a [`Gauge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GaugeStyle {
    /// Full-thickness bar over a dim band track.
    #[default]
    Linear,
    /// Bullet chart: bands as background, thin value bar in the middle.
    Bullet,
    /// Semicircular dial drawn with braille dots.
    Arc,
}

/// Gauge showing a value inside `[min, max]`.
#[derive(Debug, Clone)]
pub struct Gauge<'a> {
    value: f64,
    min: f64,
    max: f64,
    bands: Vec<(f64, Color)>,
    target: Option<f64>,
    label: Option<String>,
    orientation: GaugeOrientation,
    style: GaugeStyle,
    color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    marker: Marker,
    block: Option<Block<'a>>,
}

impl<'a> Gauge<'a> {
    /// Create a gauge for `value` on the scale 0..=100.
    pub fn new(value: f64) -> Self {
        Self {
            value,
            min: 0.0,
            max: 100.0,
            bands: Vec::new(),
            target: None,
            label: None,
            orientation: GaugeOrientation::Horizontal,
            style: GaugeStyle::Linear,
            color: None,
            theme: None,
            base: None,
            marker: Marker::Braille,
            block: None,
        }
    }
    /// Scale range.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// Threshold bands: each `(upper_bound, color)` covers up to that value.
    pub fn bands(mut self, bands: impl IntoIterator<Item = (f64, Color)>) -> Self {
        self.bands = bands.into_iter().collect();
        self.bands.sort_by(|a, b| a.0.total_cmp(&b.0));
        self
    }
    opt_setter!(
        /// Target marker value.
        target: f64
    );
    /// Centered label text.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    setter!(
        /// Horizontal or vertical.
        orientation: GaugeOrientation
    );
    setter!(
        /// Gauge form.
        style: GaugeStyle
    );
    opt_setter!(
        /// Bar color when no bands are set.
        color: Color
    );
    theme_setter!(
        /// Apply a [`Theme`]: bar color (bull color), track/needle colors and
        /// background fill. Explicit [`Gauge::color`] and band colors win.
        theme
    );
    base_style_setter!(base);
    fn fg(&self) -> Color {
        self.theme.as_ref().map_or(Color::White, |t| t.foreground)
    }
    fn track(&self) -> Color {
        self.theme.as_ref().map_or(Color::DarkGray, |t| t.grid)
    }
    fn bar_color(&self) -> Color {
        self.color
            .unwrap_or_else(|| self.theme.as_ref().map_or(Color::Green, |t| t.bull))
    }
    setter!(
        /// Glyph family used for dot drawing (default [`Marker::Braille`]).
        marker: Marker
    );

    block_setter!(block, 'a);

    fn frac(&self, v: f64) -> f64 {
        let span = self.max - self.min;
        if !v.is_finite() || !span.is_finite() || span <= 0.0 {
            return 0.0;
        }
        ((v - self.min) / span).clamp(0.0, 1.0)
    }

    fn band_color(&self, f: f64) -> Color {
        let v = self.min + f * (self.max - self.min);
        self.bands
            .iter()
            .find(|b| v <= b.0)
            .or(self.bands.last())
            .map_or(self.bar_color(), |b| b.1)
    }

    fn draw_label(&self, area: Rect, buf: &mut Buffer) {
        let Some(l) = &self.label else { return };
        let n = l.chars().count().min(area.width as usize);
        let x0 = area.x + (area.width as usize - n) as u16 / 2;
        let y = area.y + area.height / 2;
        for (i, ch) in l.chars().take(n).enumerate() {
            if let Some(c) = buf.cell_mut((x0 + i as u16, y)) {
                c.set_char(ch).set_fg(self.fg());
            }
        }
    }

    fn draw_linear(&self, area: Rect, buf: &mut Buffer) {
        let horiz = self.orientation == GaugeOrientation::Horizontal;
        let (len, cross) = if horiz {
            (area.width, area.height)
        } else {
            (area.height, area.width)
        };
        let bullet = self.style == GaugeStyle::Bullet;
        let eighths = (self.frac(self.value) * len as f64 * 8.0).round() as i64;
        let (c0, c1) = if bullet && cross >= 3 {
            (cross / 3, cross - cross / 3)
        } else if bullet {
            (cross / 2, cross / 2 + 1)
        } else {
            (0, cross)
        };
        let tpos = self
            .target
            .map(|t| ((self.frac(t) * len as f64).floor() as u16).min(len.saturating_sub(1)));
        for i in 0..len {
            let f = (i as f64 + 0.5) / len as f64;
            let bc = self.band_color(f);
            let rem = eighths - i as i64 * 8;
            for c in 0..cross {
                let (x, y) = if horiz {
                    (area.x + i, area.y + c)
                } else {
                    (area.x + c, area.y + area.height - 1 - i)
                };
                let Some(cell) = buf.cell_mut((x, y)) else {
                    continue;
                };
                let in_bar = c >= c0 && c < c1;
                if tpos == Some(i) {
                    cell.set_char(if horiz { '┃' } else { '━' })
                        .set_fg(self.fg());
                } else if in_bar && rem >= 8 {
                    cell.set_char('█').set_fg(bc);
                } else if in_bar && rem > 0 {
                    let k = (rem - 1) as usize;
                    let ch = if horiz {
                        ['▏', '▎', '▍', '▌', '▋', '▊', '▉'][k]
                    } else {
                        ['▁', '▂', '▃', '▄', '▅', '▆', '▇'][k]
                    };
                    cell.set_char(ch).set_fg(bc);
                } else if !self.bands.is_empty() || bullet {
                    cell.set_char(if bullet { '▒' } else { '░' }).set_fg(bc);
                } else {
                    cell.set_char('░').set_fg(self.track());
                }
            }
        }
        self.draw_label(area, buf);
    }

    fn draw_arc(&self, area: Rect, buf: &mut Buffer) {
        let mut cv = Canvas::new(area.width, area.height, self.marker);
        let (pw, ph) = (cv.px_w(), cv.px_h());
        let r = ((pw / 2).min(ph - 1)).max(1) as f64;
        let (cx, cy) = (pw as f64 / 2.0 - 0.5, ph as f64 - 1.0);
        let at = |f: f64, rad: f64| {
            let a = std::f64::consts::PI * (1.0 - f);
            (
                (cx + rad * a.cos()).round() as i32,
                (cy - rad * a.sin()).round() as i32,
            )
        };
        let vf = self.frac(self.value);
        let steps = (std::f64::consts::PI * r * 2.0).ceil() as usize + 1;
        for s in 0..=steps {
            let f = s as f64 / steps as f64;
            let col = if f <= vf || !self.bands.is_empty() {
                self.band_color(f)
            } else {
                self.track()
            };
            let col = if f > vf { self.track() } else { col };
            for t in 0..(r as i32 / 4).max(1) {
                let p = at(f, r - t as f64);
                cv.set(p.0, p.1, col);
            }
        }
        if let Some(t) = self.target {
            let tf = self.frac(t);
            let (a, b) = (at(tf, r * 0.5), at(tf, r));
            cv.line(a.0, a.1, b.0, b.1, self.fg());
        }
        let tip = at(vf, r * 0.85);
        cv.line(cx.round() as i32, cy as i32, tip.0, tip.1, self.fg());
        cv.render(buf, area);
        self.draw_label(area, buf);
    }
}

impl Gauge<'_> {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        match self.style {
            GaugeStyle::Arc => self.draw_arc(inner, buf),
            _ => self.draw_linear(inner, buf),
        }
    }
}

widget_impls!(Gauge<'_>);

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn run(g: Gauge, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        g.render(a, &mut b);
        b
    }

    #[test]
    fn half_filled() {
        let b = run(Gauge::new(50.0), 10, 1);
        assert_eq!(b[(0, 0)].symbol(), "█");
        assert_eq!(b[(4, 0)].symbol(), "█");
        assert_eq!(b[(5, 0)].symbol(), "░");
    }

    #[test]
    fn bands_target_label() {
        let g = Gauge::new(80.0)
            .bands([(50.0, Color::Green), (100.0, Color::Red)])
            .target(25.0)
            .label("ab");
        let b = run(g, 20, 3);
        assert_eq!(b[(5, 0)].symbol(), "┃");
        assert_eq!(b[(0, 0)].fg, Color::Green);
        assert_eq!(b[(15, 0)].fg, Color::Red);
        assert_eq!(b[(9, 1)].symbol(), "a");
    }

    #[test]
    fn vertical_fills_from_bottom() {
        let b = run(
            Gauge::new(100.0).orientation(GaugeOrientation::Vertical),
            2,
            4,
        );
        assert_eq!(b[(0, 3)].symbol(), "█");
        assert_eq!(b[(0, 0)].symbol(), "█");
        let b = run(
            Gauge::new(0.0).orientation(GaugeOrientation::Vertical),
            2,
            4,
        );
        assert_eq!(b[(0, 3)].symbol(), "░");
    }

    #[test]
    fn bullet_and_arc_and_degenerate() {
        let b = run(Gauge::new(60.0).style(GaugeStyle::Bullet), 10, 3);
        assert_eq!(b[(0, 1)].symbol(), "█");
        assert_eq!(b[(0, 0)].symbol(), "▒");
        let b = run(Gauge::new(60.0).style(GaugeStyle::Arc).target(30.0), 20, 6);
        assert!(b.content().iter().any(|c| c.symbol() != " "));
        for s in [GaugeStyle::Linear, GaugeStyle::Bullet, GaugeStyle::Arc] {
            run(
                Gauge::new(f64::NAN)
                    .style(s)
                    .range(1.0, 1.0)
                    .target(f64::NAN),
                1,
                1,
            );
            run(Gauge::new(5.0).style(s).label("long label"), 3, 2);
            run(Gauge::new(5.0).style(s), 0, 0);
        }
    }
}
