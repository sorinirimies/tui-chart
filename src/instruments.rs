//! Instrument-style charts: [`Oscilloscope`], [`Spectrum`], [`DotMatrix`],
//! [`RadarScope`] and [`HexMap`].
//!
//! The widgets are stateless: animated ones take a `phase` (`0..1`, wrapping)
//! that the caller advances every frame.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::braille::{polar, Braille};
use crate::theme::{self, Theme};
use crate::util::{darken, mix};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};
use std::f64::consts::TAU;

// ───────────────────────────── pure helpers ─────────────────────────────

/// One heartbeat-shaped period (`-1..=1`) for `x` in `0..1`: P wave, QRS spike, T wave.
pub fn ecg_beat(x: f64) -> f64 {
    let g = |c: f64, w: f64, a: f64| a * (-((x - c) / w).powi(2)).exp();
    g(0.18, 0.04, 0.18)
        + g(0.36, 0.012, -0.22)
        + g(0.39, 0.012, 1.0)
        + g(0.42, 0.012, -0.35)
        + g(0.62, 0.06, 0.28)
}

/// Brightness `0..=1` of a radar blip at turn-fraction `blip` while the sweep is at
/// `sweep`: full as the beam passes, fading over one revolution.
pub fn blip_alpha(sweep: f64, blip: f64) -> f64 {
    let since = (sweep - blip).rem_euclid(1.0);
    (1.0 - since).powi(2)
}

/// Centre of hex `(row, col)` for a pointy-top grid with circumradius `s`.
pub fn hex_center(row: usize, col: usize, s: f64) -> (f64, f64) {
    let w = 3f64.sqrt() * s;
    let off = if row % 2 == 1 { w / 2.0 } else { 0.0 };
    (w / 2.0 + col as f64 * w + off, s + row as f64 * 1.5 * s)
}

/// Grid size `(width, height)` of `rows × cols` hexes of circumradius `s`.
pub fn hex_extent(rows: usize, cols: usize, s: f64) -> (f64, f64) {
    let w = 3f64.sqrt() * s;
    (w * (cols as f64 + 0.5), s * (1.5 * rows as f64 + 0.5))
}

/// Whether `(dx, dy)` (relative to the center) lies inside a pointy-top hex of circumradius `r`.
fn in_hex(dx: f64, dy: f64, r: f64) -> bool {
    let (dx, dy) = (dx.abs(), dy.abs());
    dx <= 3f64.sqrt() / 2.0 * r && dy <= r - dx / 3f64.sqrt()
}

/// Animated bar value: `v` modulated by a per-bar sine at time `t`.
pub fn jitter(v: f64, i: usize, t: f64) -> f64 {
    let f = 1.7 + i as f64 * 0.37;
    (v * (0.65 + 0.35 * (t * f + i as f64).sin())).clamp(0.0, 1.0)
}

fn unit(v: f64) -> f64 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Colors shared by the instruments.
struct Pal {
    primary: Color,
    danger: Color,
    ok: Color,
    warn: Color,
    grid: Color,
    fg: Color,
}

fn pal(theme: &Option<Theme>, color: Option<Color>) -> Pal {
    match theme {
        Some(t) => Pal {
            primary: color.unwrap_or(t.accent),
            danger: t.bear,
            ok: t.bull,
            warn: t.neutral,
            grid: t.grid,
            fg: t.foreground,
        },
        None => Pal {
            primary: color.unwrap_or(Color::Cyan),
            danger: Color::Red,
            ok: Color::Green,
            warn: Color::Yellow,
            grid: Color::DarkGray,
            fg: Color::White,
        },
    }
}

fn put(buf: &mut Buffer, x: u16, y: u16, ch: char, style: Style) {
    if let Some(c) = buf.cell_mut((x, y)) {
        c.set_char(ch).set_style(style);
    }
}

/// Declares the shared `theme` / `base_style` / `block` builders and trait impls.
macro_rules! instrument_common {
    ($t:ty) => {
        impl $t {
            theme_setter!(
                /// Apply a [`Theme`]; an explicit `.color(..)` wins.
                theme
            );
            base_style_setter!(base);
            block_setter!(block, 'static);
        }
        widget_impls!($t);
        #[cfg(feature = "theme")]
        impl tui_nuances::Themed for $t {
            fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
                self.theme(theme)
            }
        }
    };
}

// ────────────────────────────── Oscilloscope ──────────────────────────────

/// Phosphor-style trace over a divided graticule. Samples are `-1..=1`.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::Oscilloscope;
///
/// let scope = Oscilloscope::ecg(3, 240).phase(0.25);
/// let area = Rect::new(0, 0, 60, 16);
/// let mut buf = Buffer::empty(area);
/// scope.render(area, &mut buf);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Oscilloscope {
    samples: Vec<f64>,
    phase: f64,
    grid: bool,
    color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl Oscilloscope {
    /// A trace through `samples` (`-1..=1`; other values are clamped).
    pub fn new(samples: impl IntoIterator<Item = f64>) -> Self {
        Self {
            samples: samples.into_iter().collect(),
            phase: 0.0,
            grid: true,
            ..Self::default()
        }
    }

    /// A heartbeat: `beats` periods over `n` samples.
    pub fn ecg(beats: usize, n: usize) -> Self {
        let (beats, n) = (beats.max(1) as f64, n.max(2));
        Self::new((0..n).map(|i| ecg_beat((i as f64 / n as f64 * beats).fract())))
    }

    /// A sum of sines: `(frequency, amplitude)` partials over `n` samples.
    pub fn wave(partials: &[(f64, f64)], n: usize) -> Self {
        let n = n.max(2);
        let norm = partials.iter().map(|p| p.1.abs()).sum::<f64>().max(1e-3);
        Self::new((0..n).map(|i| {
            let x = i as f64 / n as f64;
            partials
                .iter()
                .map(|&(f, a)| a * (x * f * TAU).sin())
                .sum::<f64>()
                / norm
        }))
    }

    setter!(
        /// Scroll position `0..1` (wraps): the trace is shifted by this fraction.
        phase: f64
    );
    setter!(
        /// Show the graticule (default on).
        grid: bool
    );
    opt_setter!(
        /// Trace color.
        color: Color
    );

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.is_empty() {
            return;
        }
        let p = pal(&self.theme, self.color);
        let mut g = Braille::new(inner.width, inner.height);
        let (w, h) = (g.dots_w(), g.dots_h());
        if self.grid {
            let major = mix(p.grid, p.fg, 0.4);
            for i in 0..=10 {
                let x = ((w - 1.0) * f64::from(i) / 10.0).round();
                let (col, step) = if i == 5 {
                    (major, 1)
                } else {
                    (darken(p.grid, 0.4), 8)
                };
                let mut y = 0.0;
                while y < h {
                    g.plot(x, y, col, 0);
                    y += f64::from(step);
                }
            }
            for i in 0..=8 {
                let y = ((h - 1.0) * f64::from(i) / 8.0).round();
                let (col, step) = if i == 4 {
                    (major, 1)
                } else {
                    (darken(p.grid, 0.4), 8)
                };
                let mut x = 0.0;
                while x < w {
                    g.plot(x, y, col, 0);
                    x += f64::from(step);
                }
            }
        }
        let n = self.samples.len();
        if n >= 2 {
            let shift = (self.phase.rem_euclid(1.0) * n as f64) as usize;
            let pts: Vec<(f64, f64)> = (0..n)
                .map(|i| {
                    let s = self.samples[(i + shift) % n];
                    let s = if s.is_finite() {
                        s.clamp(-1.0, 1.0)
                    } else {
                        0.0
                    };
                    (
                        (w - 1.0) * i as f64 / (n - 1) as f64,
                        (h - 1.0) / 2.0 - s * (h - 1.0) * 0.45,
                    )
                })
                .collect();
            if crate::interact::recording() {
                for cx in 0..inner.width {
                    let i = ((f64::from(cx) + 0.5) / f64::from(inner.width) * (n - 1) as f64)
                        .round() as usize;
                    let i = i.min(n - 1);
                    let cy = (pts[i].1 / 4.0).max(0.0) as u16;
                    let rect = Rect::new(inner.x + cx, inner.y + cy, 1, 1).intersection(inner);
                    crate::interact::record_point(rect, 0, i, || {
                        let s = self.samples[(i + shift) % n];
                        (format!("sample {i}"), vec![format!("{s:.2}")])
                    });
                }
            }
            let halo = darken(p.primary, 0.6);
            for pair in pts.windows(2) {
                for dy in [-1.0, 1.0] {
                    g.line(
                        (pair[0].0, pair[0].1 + dy),
                        (pair[1].0, pair[1].1 + dy),
                        halo,
                        1,
                    );
                }
                g.line(pair[0], pair[1], p.primary, 2);
            }
        }
        g.blit(inner, buf);
    }
}

instrument_common!(Oscilloscope);

// ──────────────────────────────── Spectrum ────────────────────────────────

const V_EIGHTHS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Equalizer bars (`0..=1`) built from segments with zone colors and peak caps.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::Spectrum;
///
/// let s = Spectrum::new([0.2, 0.8, 0.5, 0.95, 0.4]).peaks([0.3, 0.9, 0.6, 1.0, 0.5]);
/// let area = Rect::new(0, 0, 30, 10);
/// let mut buf = Buffer::empty(area);
/// s.render(area, &mut buf);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Spectrum {
    values: Vec<f64>,
    peaks: Vec<f64>,
    zones: bool,
    color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl Spectrum {
    /// Bars with `values` in `0..=1`.
    pub fn new(values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            values: values.into_iter().collect(),
            zones: true,
            ..Self::default()
        }
    }

    /// Peak-hold caps (`0..=1`), one per bar.
    pub fn peaks(mut self, peaks: impl IntoIterator<Item = f64>) -> Self {
        self.peaks = peaks.into_iter().collect();
        self
    }

    setter!(
        /// Color the upper segments green / yellow / red (default on).
        zones: bool
    );
    opt_setter!(
        /// Bar color when zones are off.
        color: Color
    );

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        let n = self.values.len();
        if inner.is_empty() || n == 0 {
            return;
        }
        let p = pal(&self.theme, self.color);
        let width = usize::from(inner.width);
        let (mut bw, mut gap) = (((width + 1) / n).saturating_sub(1).max(1), 1);
        if n * bw + (n - 1) * gap > width {
            bw = 1;
            gap = usize::from(n * 2 - 1 <= width);
        }
        let shown = n.min((width + gap) / (bw + gap));
        let used = shown * bw + shown.saturating_sub(1) * gap;
        let x0 = inner.x + ((width - used) / 2) as u16;
        let rows = usize::from(inner.height);
        let zone_color = |frac: f64| {
            if !self.zones {
                p.primary
            } else if frac < 0.6 {
                p.ok
            } else if frac < 0.85 {
                p.warn
            } else {
                p.danger
            }
        };
        for (i, v) in self.values.iter().take(shown).enumerate() {
            let eighths = (unit(*v) * rows as f64 * 8.0).round() as i64;
            let peak_row = self
                .peaks
                .get(i)
                .map(|p| ((unit(*p) * rows as f64).ceil() as usize).clamp(1, rows) - 1);
            let bx = x0 + (i * (bw + gap)) as u16;
            crate::interact::record_point(
                Rect::new(bx, inner.y, bw as u16, inner.height),
                0,
                i,
                || {
                    let mut lines = vec![format!("{:.0}%", unit(*v) * 100.0)];
                    if let Some(pk) = self.peaks.get(i) {
                        lines.push(format!("peak {:.0}%", unit(*pk) * 100.0));
                    }
                    (format!("#{i}"), lines)
                },
            );
            for r in 0..rows {
                let frac = (r as f64 + 0.5) / rows as f64;
                let col = zone_color(frac);
                let rem = eighths - r as i64 * 8;
                let y = inner.bottom() - 1 - r as u16;
                let (ch, style) = if rem >= 8 {
                    ('█', Style::default().fg(col))
                } else if rem > 0 {
                    (V_EIGHTHS[rem as usize], Style::default().fg(col))
                } else if peak_row == Some(r) && unit(self.peaks[i]) > 0.0 {
                    ('▀', Style::default().fg(col))
                } else {
                    ('░', Style::default().fg(darken(col, 0.75)))
                };
                for dx in 0..bw {
                    put(buf, bx + dx as u16, y, ch, style);
                }
            }
        }
    }
}

instrument_common!(Spectrum);

// ─────────────────────────────── DotMatrix ────────────────────────────────

/// LED dot matrix: one column of dots per value (`0..=1`), lit from the bottom.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::DotMatrix;
///
/// let m = DotMatrix::new([0.1, 0.4, 0.9, 0.6]).rows(8);
/// let area = Rect::new(0, 0, 20, 8);
/// let mut buf = Buffer::empty(area);
/// m.render(area, &mut buf);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DotMatrix {
    values: Vec<f64>,
    rows: Option<usize>,
    color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl DotMatrix {
    /// Columns with `values` in `0..=1`.
    pub fn new(values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            values: values.into_iter().collect(),
            ..Self::default()
        }
    }

    /// Number of dot rows (default: the area height; at most the area height).
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = Some(rows.max(2));
        self
    }

    opt_setter!(
        /// Dot color.
        color: Color
    );

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        let n = self.values.len();
        if inner.is_empty() || n == 0 {
            return;
        }
        let p = pal(&self.theme, self.color);
        let width = usize::from(inner.width);
        let pitch = if n * 2 - 1 <= width { 2 } else { 1 };
        let shown = n.min(width.div_ceil(pitch));
        let used = (shown - 1) * pitch + 1;
        let x0 = inner.x + ((width - used) / 2) as u16;
        let rows = self
            .rows
            .unwrap_or(usize::MAX)
            .min(usize::from(inner.height));
        let dim = Style::default().fg(darken(p.grid, 0.2));
        let on = Style::default().fg(p.primary);
        let soft = Style::default().fg(mix(p.grid, p.primary, 0.55));
        for (i, v) in self.values.iter().take(shown).enumerate() {
            let f = unit(*v);
            crate::interact::record_point(
                Rect::new(x0 + (i * pitch) as u16, inner.y, 1, inner.height),
                0,
                i,
                || (format!("#{i}"), vec![format!("{:.0}%", f * 100.0)]),
            );
            for k in 0..rows {
                let cov = (f * rows as f64 - k as f64).clamp(0.0, 1.0);
                let (ch, st) = if cov >= 0.99 {
                    ('●', on)
                } else if cov > 0.0 {
                    ('•', soft)
                } else {
                    ('·', dim)
                };
                put(
                    buf,
                    x0 + (i * pitch) as u16,
                    inner.bottom() - 1 - k as u16,
                    ch,
                    st,
                );
            }
        }
    }
}

instrument_common!(DotMatrix);

// ─────────────────────────────── RadarScope ───────────────────────────────

/// A contact on a [`RadarScope`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Blip {
    /// Bearing in turns: `0` = up, clockwise, wraps.
    pub bearing: f64,
    /// Distance from the center, `0..=1` of the radius.
    pub range: f64,
}

impl Blip {
    /// A blip at `bearing` (turns) and `range` (`0..=1`).
    pub fn new(bearing: f64, range: f64) -> Self {
        Self { bearing, range }
    }
}

/// Radar screen: range rings, a rotating beam with a fading wedge and blips
/// that light up as the beam passes.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{Blip, RadarScope};
///
/// let scope = RadarScope::new([Blip::new(0.1, 0.6), Blip::new(0.7, 0.3)]).phase(0.2);
/// let area = Rect::new(0, 0, 40, 20);
/// let mut buf = Buffer::empty(area);
/// scope.render(area, &mut buf);
/// ```
#[derive(Debug, Clone, Default)]
pub struct RadarScope {
    blips: Vec<Blip>,
    phase: f64,
    fade: bool,
    color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl RadarScope {
    /// A scope showing `blips`.
    pub fn new(blips: impl IntoIterator<Item = Blip>) -> Self {
        Self {
            blips: blips.into_iter().collect(),
            phase: 0.12,
            fade: true,
            ..Self::default()
        }
    }

    setter!(
        /// Beam position in turns (`0` = up, clockwise, wraps).
        phase: f64
    );
    setter!(
        /// Fade blips with the beam (default on); off keeps them fully lit.
        fade: bool
    );
    opt_setter!(
        /// Beam and ring color.
        color: Color
    );

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.is_empty() {
            return;
        }
        let p = pal(&self.theme, self.color);
        let mut g = Braille::new(inner.width, inner.height);
        let (w, h) = (g.dots_w(), g.dots_h());
        let c = (w / 2.0, h / 2.0);
        let r = w.min(h) / 2.0 - 1.0;
        if r < 4.0 {
            return;
        }
        for i in 1..=4 {
            let ro = r * f64::from(i) / 4.0;
            g.ring(c, (ro - 0.6, ro), (0.0, 1.0), |_| Some((p.grid, 0)));
        }
        for q in 0..4 {
            g.line(c, polar(c.0, c.1, r, f64::from(q) / 4.0), p.grid, 0);
        }
        let tick = darken(p.primary, 0.45);
        for i in 0..36 {
            let a = f64::from(i) / 36.0;
            let len = if i % 3 == 0 { 5.0 } else { 2.5 };
            g.line(polar(c.0, c.1, r, a), polar(c.0, c.1, r - len, a), tick, 1);
        }
        let sweep = self.phase.rem_euclid(1.0);
        let steps = 56;
        for k in 0..steps {
            let a = sweep - 0.22 * f64::from(k) / f64::from(steps);
            let t = 1.0 - f64::from(k) / f64::from(steps);
            g.line(
                c,
                polar(c.0, c.1, r, a),
                darken(p.primary, 1.0 - 0.5 * t * t),
                2,
            );
        }
        g.line(c, polar(c.0, c.1, r, sweep), p.primary, 3);
        for (ri, b) in self.blips.iter().enumerate() {
            let a = if self.fade {
                blip_alpha(sweep, b.bearing.rem_euclid(1.0))
            } else {
                1.0
            };
            if a < 0.04 {
                continue;
            }
            let (x, y) = polar(c.0, c.1, r * unit(b.range), b.bearing);
            let (bi, rect) = (
                ri,
                Rect::new(
                    inner.x + (x.max(0.0) / 2.0) as u16,
                    inner.y + (y.max(0.0) / 4.0) as u16,
                    1,
                    1,
                )
                .intersection(inner),
            );
            crate::interact::record_point(rect, 0, bi, || {
                (
                    format!("contact #{bi}"),
                    vec![
                        format!("bearing {:.0}°", b.bearing.rem_euclid(1.0) * 360.0),
                        format!("range {:.0}%", unit(b.range) * 100.0),
                    ],
                )
            });
            let col = darken(p.danger, 1.0 - a);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    g.plot(x + f64::from(dx), y + f64::from(dy), col, 4);
                }
            }
        }
        g.plot(c.0, c.1, p.primary, 5);
        g.blit(inner, buf);
    }
}

instrument_common!(RadarScope);

// ─────────────────────────────────── HexMap ───────────────────────────────

/// Honeycomb heat map: `cols` hexes per row, each colored by its value (`0..=1`).
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::HexMap;
///
/// let m = HexMap::new(6, (0..24).map(|i| f64::from(i) / 23.0));
/// let area = Rect::new(0, 0, 40, 16);
/// let mut buf = Buffer::empty(area);
/// m.render(area, &mut buf);
/// ```
#[derive(Debug, Clone, Default)]
pub struct HexMap {
    values: Vec<f64>,
    cols: usize,
    phase: Option<f64>,
    color: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl HexMap {
    /// A honeycomb with `cols` hexes per row, filled row by row from `values`.
    pub fn new(cols: usize, values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            values: values.into_iter().collect(),
            cols: cols.max(1),
            ..Self::default()
        }
    }

    /// Pulse the cells with a phase (`0..1`, wraps); `None` is static.
    pub fn phase(mut self, phase: f64) -> Self {
        self.phase = Some(phase);
        self
    }

    opt_setter!(
        /// Color of hot cells.
        color: Color
    );

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.is_empty() || self.values.is_empty() {
            return;
        }
        let p = pal(&self.theme, self.color);
        let mut g = Braille::new(inner.width, inner.height);
        let (w, h) = (g.dots_w(), g.dots_h());
        let rows = self.values.len().div_ceil(self.cols);
        let s = (w / (3f64.sqrt() * (self.cols as f64 + 0.5))).min(h / (1.5 * rows as f64 + 0.5));
        if s < 2.0 {
            return;
        }
        let (gw, gh) = hex_extent(rows, self.cols, s);
        let (ox, oy) = ((w - gw) / 2.0, (h - gh) / 2.0);
        let base = mix(Color::Black, p.grid, 0.5);
        for (i, v) in self.values.iter().enumerate() {
            let (row, col) = (i / self.cols, i % self.cols);
            let mut v = unit(*v);
            if let Some(ph) = self.phase {
                v = (v * (0.75 + 0.25 * (ph * TAU + i as f64 * 0.9).sin())).clamp(0.0, 1.0);
            }
            let color = if v > 0.8 {
                mix(p.primary, p.danger, (v - 0.8) / 0.2)
            } else {
                mix(base, p.primary, v / 0.8)
            };
            let (cx, cy) = hex_center(row, col, s);
            let (cx, cy) = (cx + ox, cy + oy);
            let r = s * 0.9;
            let hw = ((3f64.sqrt() * s / 4.0).round() as u16).max(1);
            let rect = Rect::new(
                (inner.x + (cx / 2.0).max(0.0) as u16).saturating_sub(hw / 2),
                inner.y + (cy / 4.0).max(0.0) as u16,
                hw,
                1,
            )
            .intersection(inner);
            crate::interact::record_point(rect, 0, i, || {
                (
                    format!("hex ({row}, {col})"),
                    vec![format!("{:.0}%", v * 100.0)],
                )
            });
            for py in (cy - r).floor() as i64..=(cy + r).ceil() as i64 {
                for px in (cx - r).floor() as i64..=(cx + r).ceil() as i64 {
                    if in_hex(px as f64 + 0.5 - cx, py as f64 + 0.5 - cy, r) {
                        g.plot(px as f64, py as f64, color, 1);
                    }
                }
            }
        }
        g.blit(inner, buf);
    }
}

instrument_common!(HexMap);

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn draw(w: impl Widget, width: u16, height: u16) -> Buffer {
        let a = Rect::new(0, 0, width, height);
        let mut b = Buffer::empty(a);
        w.render(a, &mut b);
        b
    }

    fn painted(b: &Buffer) -> usize {
        b.content().iter().filter(|c| c.symbol() != " ").count()
    }

    #[test]
    fn ecg_peaks_at_the_qrs_spike() {
        assert!(ecg_beat(0.39) > 0.9);
        assert!(ecg_beat(0.9).abs() < 0.05);
    }

    #[test]
    fn blip_alpha_is_full_at_the_beam_and_fades() {
        assert!((blip_alpha(0.5, 0.5) - 1.0).abs() < 1e-12);
        assert!(blip_alpha(0.5, 0.4) > blip_alpha(0.5, 0.2));
        assert!(blip_alpha(0.1, 0.9) > 0.0);
    }

    #[test]
    fn hex_geometry() {
        let (x, y) = hex_center(0, 0, 10.0);
        assert!((x - 3f64.sqrt() * 5.0).abs() < 1e-9 && (y - 10.0).abs() < 1e-9);
        assert!(hex_center(1, 0, 10.0).0 > x);
        assert!(in_hex(0.0, 0.0, 5.0) && !in_hex(5.0, 0.0, 5.0) && !in_hex(0.0, 5.1, 5.0));
    }

    #[test]
    fn jitter_stays_in_unit_range() {
        for i in 0..8 {
            for t in [0.0, 1.0, 55.5] {
                assert!((0.0..=1.0).contains(&jitter(0.9, i, t)));
            }
        }
    }

    #[test]
    fn oscilloscope_draws_trace_and_grid_and_scrolls() {
        let a = draw(Oscilloscope::ecg(2, 200), 60, 16);
        assert!(painted(&a) > 100);
        let no_grid = draw(Oscilloscope::ecg(2, 200).grid(false), 60, 16);
        assert!(painted(&no_grid) < painted(&a));
        assert_ne!(a, draw(Oscilloscope::ecg(2, 200).phase(0.3), 60, 16));
        let w = Oscilloscope::wave(&[(2.0, 1.0), (5.0, 0.5)], 100);
        assert!(painted(&draw(w.color(Color::Magenta), 40, 10)) > 0);
    }

    #[test]
    fn spectrum_fills_bars_with_zone_colors() {
        let b = draw(Spectrum::new([0.2, 1.0, 0.6]), 12, 10);
        for c in [Color::Green, Color::Red] {
            assert!(
                b.content().iter().any(|x| x.fg == c && x.symbol() == "█"),
                "{c:?}"
            );
        }
        let off = draw(Spectrum::new([1.0]).zones(false).color(Color::Blue), 6, 5);
        assert!(off.content().iter().any(|x| x.fg == Color::Blue));
    }

    #[test]
    fn spectrum_peak_cap_sits_above_the_bar() {
        let b = draw(Spectrum::new([0.2]).peaks([0.9]), 3, 10);
        assert!(b.content().iter().any(|c| c.symbol() == "▀"));
    }

    #[test]
    fn dot_matrix_lights_from_the_bottom() {
        let b = draw(DotMatrix::new([1.0, 0.0]), 3, 4);
        assert_eq!(b[(0, 3)].symbol(), "●");
        assert_eq!(b[(0, 0)].symbol(), "●");
        assert_eq!(b[(2, 3)].symbol(), "·");
        let half = draw(DotMatrix::new([0.5]), 1, 4);
        assert_eq!(half[(0, 3)].symbol(), "●");
        assert_eq!(half[(0, 0)].symbol(), "·");
    }

    #[test]
    fn radar_scope_moves_with_phase_and_shows_blips() {
        let s = |ph: f64| RadarScope::new([Blip::new(0.25, 0.7)]).phase(ph);
        let (a, b) = (draw(s(0.26), 40, 20), draw(s(0.8), 40, 20));
        assert!(painted(&a) > 50);
        assert_ne!(a, b);
        assert!(a
            .content()
            .iter()
            .any(|c| c.fg != Color::Reset && c.symbol() != " "));
        let steady = draw(s(0.8).fade(false), 40, 20);
        assert_ne!(steady, b);
    }

    #[test]
    fn hex_map_colors_follow_values() {
        let cold = draw(HexMap::new(4, [0.0; 12]), 30, 12);
        let hot = draw(HexMap::new(4, [1.0; 12]).color(Color::Cyan), 30, 12);
        assert_ne!(cold, hot);
        assert!(hot.content().iter().any(|c| matches!(c.fg, Color::Rgb(..))));
        let pulsing = draw(
            HexMap::new(4, [1.0; 12]).color(Color::Cyan).phase(0.4),
            30,
            12,
        );
        assert_ne!(pulsing, hot);
    }

    #[test]
    fn all_instruments_survive_odd_input_and_sizes() {
        let odd = [f64::NAN, f64::INFINITY, -3.0, 0.5, 1e300];
        for (w, h) in [(0, 0), (1, 1), (2, 2), (5, 3), (17, 9), (80, 30)] {
            for blk in [false, true] {
                let b = || blk.then(Block::bordered);
                let mut o = Oscilloscope::new(odd).phase(f64::NAN);
                let mut s = Spectrum::new(odd.iter().cycle().take(40).copied()).peaks(odd);
                let mut d = DotMatrix::new(odd).rows(99);
                let mut r = RadarScope::new([Blip::new(f64::NAN, f64::NAN), Blip::new(9.0, -1.0)])
                    .phase(f64::INFINITY);
                let mut m = HexMap::new(0, odd).phase(f64::NAN);
                if let Some(bl) = b() {
                    o = o.block(bl.clone());
                    s = s.block(bl.clone());
                    d = d.block(bl.clone());
                    r = r.block(bl.clone());
                    m = m.block(bl);
                }
                draw(o, w, h);
                draw(s, w, h);
                draw(d, w, h);
                draw(r, w, h);
                draw(m, w, h);
            }
            draw(Oscilloscope::new([]), w, h);
            draw(Spectrum::new([]), w, h);
            draw(DotMatrix::new([]), w, h);
            draw(RadarScope::new([]), w, h);
            draw(HexMap::new(3, []), w, h);
            draw(HexMap::new(1000, [1.0; 5]), w, h);
        }
    }
}
