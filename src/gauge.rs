//! Gauge: linear / bullet / arc gauge with threshold bands and target marker.

use crate::canvas::{Canvas, Marker};
use crate::theme::{self, Theme};
use crate::util::{darken, format_compact};
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
    /// Semicircular dial drawn with braille dots: banded ring, ticks, needle,
    /// target notch, min/max scale labels and the value label underneath.
    Arc,
    /// Discrete blocks (htop / battery style); lit blocks take the band color.
    Segmented,
    /// Full-circle-ish dial (270°): banded ring, ticks, needle and value readout.
    Dial,
    /// Half-circle meter with a needle and the readout underneath.
    Meter,
    /// Sci-fi reticle: ring with tick marks, inner dashed ring and crosshair.
    Reticle,
    /// Full ring (360°) filling clockwise, value in the hole.
    Ring,
    /// Row of chevron cells (`❯`), lit up to the value.
    Chevron,
    /// Smooth sub-cell bar with a rounded end.
    Smooth,
    /// Row of round dots, lit up to the value.
    Dots,
    /// LED strip: lit segments take the band color, unlit ones are dim.
    Led,
    /// Battery outline with a terminal nub and a charge fill.
    Battery,
    /// Vertical liquid level in a tank, with a wavy surface.
    Level,
    /// Vertical thermometer with a bulb.
    Thermometer,
    /// Rising signal bars (like a phone's reception indicator).
    Signal,
}

impl GaugeStyle {
    /// Every gauge style.
    pub const ALL: [GaugeStyle; 16] = [
        GaugeStyle::Linear,
        GaugeStyle::Bullet,
        GaugeStyle::Arc,
        GaugeStyle::Segmented,
        GaugeStyle::Dial,
        GaugeStyle::Meter,
        GaugeStyle::Reticle,
        GaugeStyle::Ring,
        GaugeStyle::Chevron,
        GaugeStyle::Smooth,
        GaugeStyle::Dots,
        GaugeStyle::Led,
        GaugeStyle::Battery,
        GaugeStyle::Level,
        GaugeStyle::Thermometer,
        GaugeStyle::Signal,
    ];

    fn look(self) -> Option<crate::gauge_kit::Look> {
        use crate::gauge_kit::Look;
        Some(match self {
            GaugeStyle::Dial => Look::Dial,
            GaugeStyle::Meter => Look::Meter,
            GaugeStyle::Reticle => Look::Reticle,
            GaugeStyle::Ring => Look::Ring,
            GaugeStyle::Chevron => Look::Chevron,
            GaugeStyle::Smooth => Look::Smooth,
            GaugeStyle::Dots => Look::Dots,
            GaugeStyle::Led => Look::Led,
            GaugeStyle::Battery => Look::Battery,
            GaugeStyle::Level => Look::Level,
            GaugeStyle::Thermometer => Look::Thermometer,
            GaugeStyle::Signal => Look::Signal,
            _ => return None,
        })
    }

    /// A good `(width, height)` in cells for this style.
    pub const fn preferred_size(self) -> (u16, u16) {
        match self {
            GaugeStyle::Meter => (22, 7),
            GaugeStyle::Level => (8, 8),
            GaugeStyle::Thermometer => (5, 8),
            GaugeStyle::Signal => (14, 6),
            GaugeStyle::Arc
            | GaugeStyle::Dial
            | GaugeStyle::Reticle
            | GaugeStyle::Ring
            | GaugeStyle::Segmented => (20, 10),
            _ => (36, 1),
        }
    }
}

/// Gauge showing a value inside `[min, max]`.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{Gauge};
///
/// let chart = Gauge::new(0.62);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
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

    /// Color of the unfilled part of the track at fraction `f`: the band color
    /// (or track color) darkened, so bands stay readable without hatching.
    fn dim(&self, f: f64) -> Color {
        let base = if self.bands.is_empty() {
            self.track()
        } else {
            self.band_color(f)
        };
        darken(base, if self.bands.is_empty() { 0.35 } else { 0.5 })
    }

    fn draw_label_at(&self, text: &str, cx: u16, y: u16, clip: Rect, buf: &mut Buffer) {
        let n = text.chars().count().min(clip.width as usize) as u16;
        let x0 = cx
            .saturating_sub(n / 2)
            .clamp(clip.x, clip.right().saturating_sub(n));
        for (i, ch) in text.chars().take(n as usize).enumerate() {
            if let Some(c) = buf.cell_mut((x0 + i as u16, y)) {
                c.set_char(ch)
                    .set_fg(self.fg())
                    .set_style(Style::default().add_modifier(ratatui::style::Modifier::BOLD));
            }
        }
    }

    fn draw_label(&self, area: Rect, buf: &mut Buffer) {
        let Some(l) = &self.label else { return };
        self.draw_label_at(
            l,
            area.x + area.width / 2,
            area.y + area.height / 2,
            area,
            buf,
        );
    }

    fn draw_linear(&self, area: Rect, buf: &mut Buffer) {
        let horiz = self.orientation == GaugeOrientation::Horizontal;
        let (len, cross) = if horiz {
            (area.width, area.height)
        } else {
            (area.height, area.width)
        };
        let bullet = self.style == GaugeStyle::Bullet;
        let segmented = self.style == GaugeStyle::Segmented;
        let vf = self.frac(self.value);
        let eighths = (vf * len as f64 * 8.0).round() as i64;
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
        // segmented: 2-cell blocks (1 vertically) separated by a 1-cell gap
        let seg = if horiz { 2 } else { 1 };
        let nseg = ((len as usize + 1) / (seg + 1)).max(1);
        for i in 0..len {
            let f = (i as f64 + 0.5) / len as f64;
            let bc = self.band_color(f);
            let track = self.dim(f);
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
                if segmented {
                    let slot = i as usize % (seg + 1);
                    if slot == seg {
                        continue; // gap
                    }
                    let idx = i as usize / (seg + 1);
                    let sf = (idx as f64 + 0.5) / nseg as f64;
                    let lit = sf <= vf || (idx == 0 && vf > 0.0);
                    let col = if lit {
                        self.band_color(sf)
                    } else {
                        self.dim(sf)
                    };
                    cell.set_char('█').set_fg(col);
                    continue;
                }
                let in_bar = c >= c0 && c < c1;
                if tpos == Some(i) {
                    cell.set_char(if horiz { '┃' } else { '━' })
                        .set_fg(self.fg())
                        .set_bg(track);
                } else if in_bar && rem >= 8 {
                    cell.set_char('█').set_fg(bc).set_bg(bc);
                } else if in_bar && rem > 0 {
                    let k = (rem - 1) as usize;
                    let ch = if horiz {
                        ['▏', '▎', '▍', '▌', '▋', '▊', '▉'][k]
                    } else {
                        ['▁', '▂', '▃', '▄', '▅', '▆', '▇'][k]
                    };
                    cell.set_char(ch).set_fg(bc).set_bg(track);
                } else {
                    // solid dimmed track (bullet: bands behind the bar)
                    cell.set_char(' ').set_bg(track);
                }
            }
        }
        if let (Some(_), true) = (tpos, segmented) {
            // keep the target visible between blocks
            if let Some(i) = tpos {
                for c in 0..cross {
                    let (x, y) = if horiz {
                        (area.x + i, area.y + c)
                    } else {
                        (area.x + c, area.y + area.height - 1 - i)
                    };
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(if horiz { '┃' } else { '━' })
                            .set_fg(self.fg());
                    }
                }
            }
        }
        self.draw_label(area, buf);
    }

    fn draw_arc(&self, area: Rect, buf: &mut Buffer) {
        // bottom row: min · value label · max (only when there is room)
        let scale_row = area.height >= 3;
        let canvas_area = if scale_row {
            Rect::new(area.x, area.y, area.width, area.height - 1)
        } else {
            area
        };
        let mut cv = Canvas::new(canvas_area.width, canvas_area.height, self.marker);
        let (pw, ph) = (cv.px_w(), cv.px_h());
        // leave room outside the ring for tick marks
        let tick_len = if ph >= 12 { 3 } else { 0 };
        let r = (((pw - 1) / 2 - tick_len).min(ph - 1 - tick_len)).max(1) as f64;
        let thick = (r / 4.0).max(2.0);
        let (cx, cy) = ((pw - 1) as f64 / 2.0, (ph - 1) as f64);
        let vf = self.frac(self.value);
        let mid = r - thick / 2.0;
        let thumb = 1.6 / (std::f64::consts::PI * mid.max(1.0)); // ~1.6 dots wide, as a fraction
        for py in 0..ph {
            for px in 0..pw {
                let (dx, dy) = (px as f64 - cx, cy - py as f64);
                if dy < -0.5 {
                    continue;
                }
                let d = (dx * dx + dy * dy).sqrt();
                if d > r || d < r - thick {
                    continue;
                }
                let f = 1.0 - dy.max(0.0).atan2(dx) / std::f64::consts::PI;
                let col = if (f - vf).abs() <= thumb && vf > 0.0 && vf < 1.0 {
                    self.fg()
                } else if f <= vf {
                    self.band_color(f)
                } else {
                    self.dim(f)
                };
                cv.set(px, py, col);
            }
        }
        let at = |f: f64, rad: f64| {
            let a = std::f64::consts::PI * (1.0 - f);
            (
                (cx + rad * a.cos()).round() as i32,
                (cy - rad * a.sin()).round() as i32,
            )
        };
        if tick_len > 0 {
            let tick = darken(self.fg(), 0.35);
            for k in 0..=10 {
                let f = k as f64 / 10.0;
                let len = if k % 5 == 0 { tick_len as f64 } else { 1.0 };
                let (a, b) = (at(f, r + 1.0), at(f, r + 1.0 + len));
                cv.line(a.0, a.1, b.0, b.1, tick);
            }
        }
        if let Some(t) = self.target {
            let tf = self.frac(t);
            let (a, b) = (at(tf, r - thick - 1.0), at(tf, r + 1.0 + tick_len as f64));
            cv.line(a.0, a.1, b.0, b.1, self.fg());
        }
        // needle + hub
        let tip = at(vf, r - thick - 2.0);
        cv.line(cx.round() as i32, cy as i32, tip.0, tip.1, self.fg());
        let hub = 2;
        for dy in -hub..=hub {
            for dx in -hub..=hub {
                if dx * dx + dy * dy <= hub * hub {
                    cv.set(cx.round() as i32 + dx, cy as i32 + dy, self.fg());
                }
            }
        }
        cv.render(buf, canvas_area);
        if scale_row {
            let y = area.bottom() - 1;
            let (rx, _) = self.marker.resolution();
            let cell_of = |dot: f64| canvas_area.x + (dot / rx as f64).floor().max(0.0) as u16;
            self.draw_label_at(&format_compact(self.min), cell_of(cx - mid), y, area, buf);
            self.draw_label_at(&format_compact(self.max), cell_of(cx + mid), y, area, buf);
            let center = self
                .label
                .clone()
                .unwrap_or_else(|| format_compact(self.value));
            self.draw_label_at(&center, area.x + area.width / 2, y, area, buf);
        } else {
            self.draw_label(area, buf);
        }
    }
}

impl Gauge<'_> {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        crate::interact::record_point(inner, 0, 0, || {
            let f = crate::util::format_compact;
            let frac = self.frac(self.value);
            (
                "gauge".to_string(),
                vec![
                    format!("{} of {} .. {}", f(self.value), f(self.min), f(self.max)),
                    format!("{:.0}%", frac * 100.0),
                ],
            )
        });
        if let Some(look) = self.style.look() {
            let frac = self.frac(self.value);
            let span = self.max - self.min;
            let bands = self
                .bands
                .iter()
                .map(|(v, c)| (self.frac(*v), *c))
                .filter(|_| span > 0.0)
                .collect();
            let readout = self
                .label
                .clone()
                .unwrap_or_else(|| format!("{:.0}%", frac * 100.0));
            crate::gauge_kit::Painter {
                look,
                frac,
                readout,
                label: None,
                bands,
                color: self.bar_color(),
                track: self.track(),
                fg: self.fg(),
            }
            .render(inner, buf);
            return;
        }
        match self.style {
            GaugeStyle::Arc => self.draw_arc(inner, buf),
            _ => self.draw_linear(inner, buf),
        }
    }
}

widget_impls!(Gauge<'_>);

#[cfg(feature = "theme")]
impl<'a> tui_nuances::Themed for Gauge<'a> {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

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
        assert_eq!(b[(5, 0)].symbol(), " ");
        assert_ne!(b[(5, 0)].bg, Color::Reset);
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
        assert_eq!(b[(0, 3)].symbol(), " ");
    }

    #[test]
    fn segmented_lights_blocks_with_gaps() {
        let b = run(Gauge::new(50.0).style(GaugeStyle::Segmented), 12, 1);
        let s: String = (0..12).map(|x| b[(x, 0)].symbol().to_string()).collect();
        assert_eq!(&s[..], "██ ██ ██ ██ ");
        assert_ne!(b[(0, 0)].fg, b[(9, 0)].fg); // lit vs dim
    }

    #[test]
    fn arc_shows_scale_and_value() {
        let g = Gauge::new(42.0).style(GaugeStyle::Arc).range(0.0, 100.0);
        let b = run(g, 40, 12);
        let last: String = (0..40).map(|x| b[(x, 11)].symbol().to_string()).collect();
        assert!(
            last.contains('0') && last.contains("100") && last.contains("42"),
            "{last:?}"
        );
    }

    #[test]
    fn bullet_and_arc_and_degenerate() {
        let b = run(Gauge::new(60.0).style(GaugeStyle::Bullet), 10, 3);
        assert_eq!(b[(0, 1)].symbol(), "█");
        assert_eq!(b[(0, 0)].symbol(), " ");
        assert_ne!(b[(0, 0)].bg, Color::Reset);
        let b = run(Gauge::new(60.0).style(GaugeStyle::Arc).target(30.0), 20, 6);
        assert!(b.content().iter().any(|c| c.symbol() != " "));
        for s in [
            GaugeStyle::Linear,
            GaugeStyle::Bullet,
            GaugeStyle::Arc,
            GaugeStyle::Segmented,
        ] {
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

    #[test]
    fn every_style_renders_at_every_size() {
        for s in GaugeStyle::ALL {
            for (w, h) in [(0, 0), (1, 1), (3, 2), (8, 3), (22, 7), (40, 12)] {
                for v in [f64::NAN, -5.0, 0.0, 50.0, 100.0, 1e9] {
                    run(Gauge::new(v).style(s).block(Block::bordered()), w, h);
                    run(
                        Gauge::new(v)
                            .style(s)
                            .label("x")
                            .bands([(50.0, Color::Green), (100.0, Color::Red)]),
                        w,
                        h,
                    );
                }
            }
            run(Gauge::new(5.0).style(s).range(3.0, 3.0), 20, 8);
        }
    }

    #[test]
    fn new_styles_draw_and_react_to_value() {
        let lit = |b: &Buffer| {
            b.content()
                .iter()
                .map(|c| (c.symbol().to_string(), c.fg))
                .collect::<Vec<_>>()
        };
        for s in GaugeStyle::ALL.iter().skip(4) {
            let (w, h) = s.preferred_size();
            let (w, h) = (w.max(12), h.max(3));
            let lo = run(Gauge::new(10.0).style(*s).color(Color::Red), w, h);
            let hi = run(Gauge::new(90.0).style(*s).color(Color::Red), w, h);
            assert!(
                lo.content().iter().any(|c| c.symbol() != " "),
                "{s:?} blank"
            );
            assert_ne!(lit(&lo), lit(&hi), "{s:?} ignores value");
        }
    }

    #[test]
    fn bands_color_new_styles() {
        let b = run(
            Gauge::new(95.0)
                .style(GaugeStyle::Led)
                .bands([(50.0, Color::Green), (100.0, Color::Red)]),
            30,
            1,
        );
        assert!(b.content().iter().any(|c| c.fg == Color::Red));
    }
}
