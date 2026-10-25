//! Concentric progress rings: [`RadialBar`] (value / max per ring) and
//! [`RingStack`] (percentages, 270° sweep). Drawn with braille dots.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::cast_possible_wrap
)]

use crate::braille::Braille;
use crate::theme::{self, Theme};
use crate::PALETTE;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::Block,
};

fn text_width(s: &str) -> usize {
    Span::raw(s).width()
}

fn truncate(s: &str, max: usize) -> String {
    let mut out = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = text_width(ch.encode_utf8(&mut [0; 4]));
        if w + cw > max {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out
}

/// One ring of a [`RadialBar`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RadialRing {
    /// Ring label (shown in the legend).
    pub label: String,
    /// Current value.
    pub value: f64,
    /// Value at which the ring is full.
    pub max: f64,
    /// Explicit color.
    pub color: Option<Color>,
}

impl RadialRing {
    /// A ring showing `value` out of `max`.
    pub fn new(label: impl Into<String>, value: f64, max: f64) -> Self {
        Self {
            label: label.into(),
            value,
            max,
            color: None,
        }
    }

    /// Explicit ring color (default: the theme / default palette, by position).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }

    /// Fraction of the ring that is filled, `0..=1` (`0` for bad input).
    pub fn fraction(&self) -> f64 {
        if self.max.is_finite() && self.max > 0.0 && self.value.is_finite() {
            (self.value / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Sweep of a [`RadialBar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RadialSweep {
    /// 270° with the gap at the bottom (default).
    #[default]
    Arc,
    /// Full 360° rings starting at 12 o'clock.
    Full,
}

/// Concentric progress rings, outermost first, with an optional legend.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{RadialBar, RadialRing};
///
/// let chart = RadialBar::new([
///     RadialRing::new("Move", 420.0, 600.0),
///     RadialRing::new("Exercise", 25.0, 30.0),
/// ]);
/// let area = Rect::new(0, 0, 50, 14);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone, Default)]
pub struct RadialBar {
    rings: Vec<RadialRing>,
    sweep: RadialSweep,
    legend: bool,
    hidden: Vec<usize>,
    track: Option<Color>,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl RadialBar {
    /// Create a chart from rings, outermost first.
    pub fn new(rings: impl IntoIterator<Item = RadialRing>) -> Self {
        Self {
            rings: rings.into_iter().collect(),
            sweep: RadialSweep::Arc,
            legend: true,
            hidden: Vec::new(),
            track: None,
            theme: None,
            base: None,
            block: None,
        }
    }

    setter!(
        /// 270° or full sweep.
        sweep: RadialSweep
    );
    setter!(
        /// Show the legend (default on, when there is room).
        legend: bool
    );
    opt_setter!(
        /// Color of the unfilled track (default: theme grid / dark gray).
        track => track: Color
    );
    theme_setter!(
        /// Apply a [`Theme`]; explicit ring colors win.
        theme
    );
    base_style_setter!(base);
    block_setter!(block, 'static);

    /// Hide rings by index. They keep their slot and legend entry (dimmed),
    /// so they can be toggled back, see [`Interaction`](crate::Interaction).
    #[must_use]
    pub fn hidden(mut self, hidden: impl IntoIterator<Item = usize>) -> Self {
        self.hidden = hidden.into_iter().collect();
        self
    }

    fn ring_color(&self, r: &RadialRing, i: usize) -> Color {
        r.color.unwrap_or_else(|| {
            self.theme
                .as_ref()
                .map_or(PALETTE[i % PALETTE.len()], |t| t.color(i))
        })
    }

    fn track_color(&self) -> Color {
        self.track
            .unwrap_or_else(|| self.theme.as_ref().map_or(Color::DarkGray, |t| t.grid))
    }

    fn draw_rings(&self, buf: &mut Buffer, area: Rect) {
        if self.rings.is_empty() || area.is_empty() {
            return;
        }
        let track = self.track_color();
        let mut g = Braille::new(area.width, area.height);
        let (w, h) = (g.dots_w(), g.dots_h());
        let c = (w / 2.0, h / 2.0);
        let r = w.min(h) / 2.0 - 0.5;
        let n = self.rings.len() as f64;
        let pitch = (r * 0.85) / n;
        if pitch >= 1.5 {
            let thick = (pitch - 1.2).max(1.5).min(pitch);
            let sw = match self.sweep {
                RadialSweep::Arc => (0.625, 0.75),
                RadialSweep::Full => (0.0, 1.0),
            };
            for (i, ring) in self.rings.iter().enumerate() {
                let outer = r - i as f64 * pitch;
                let inner = (outer - thick).max(0.0);
                let frac = if self.hidden.contains(&i) {
                    0.0
                } else {
                    ring.fraction()
                };
                let color = self.ring_color(ring, i);
                g.ring(c, (inner, outer), sw, |pos| {
                    Some(if frac >= 1.0 || pos < frac {
                        (color, 1)
                    } else {
                        (track, 0)
                    })
                });
            }
            if crate::interact::recording() {
                let mut owner = vec![None; usize::from(area.width) * usize::from(area.height)];
                for cy in 0..area.height {
                    for cx in 0..area.width {
                        let dx = (f64::from(cx) + 0.5) * 2.0 - c.0;
                        let dy = (f64::from(cy) + 0.5) * 4.0 - c.1;
                        let d = dx.hypot(dy);
                        let turn = (dx.atan2(-dy) / std::f64::consts::TAU).rem_euclid(1.0);
                        if (turn - sw.0).rem_euclid(1.0) > sw.1 {
                            continue;
                        }
                        for i in (0..self.rings.len()).filter(|i| !self.hidden.contains(i)) {
                            let outer = r - i as f64 * pitch;
                            if d <= outer && d >= (outer - thick).max(0.0) - 1.0 {
                                owner
                                    [usize::from(cy) * usize::from(area.width) + usize::from(cx)] =
                                    Some(i);
                                break;
                            }
                        }
                    }
                }
                crate::interact::record_grid(
                    area,
                    &owner,
                    |i| i,
                    |i| {
                        let ring = &self.rings[i];
                        (
                            ring.label.clone(),
                            vec![
                                format!(
                                    "{} / {}",
                                    crate::util::format_compact(ring.value),
                                    crate::util::format_compact(ring.max)
                                ),
                                format!("{:.0}%", ring.fraction() * 100.0),
                            ],
                        )
                    },
                );
            }
        }
        g.blit(area, buf);
    }

    fn draw_legend(&self, buf: &mut Buffer, area: Rect) {
        let text = self
            .theme
            .as_ref()
            .map_or(Style::default(), |t| Style::default().fg(t.foreground));
        let label_w = area.width.saturating_sub(7) as usize;
        for (i, ring) in self.rings.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.bottom() {
                break;
            }
            let color = self.ring_color(ring, i);
            let off = self.hidden.contains(&i);
            let text = if off {
                text.add_modifier(Modifier::DIM | Modifier::CROSSED_OUT)
            } else {
                text
            };
            buf.set_string(
                area.x,
                y,
                if off { "□" } else { "■" },
                Style::default().fg(color),
            );
            let label = truncate(&ring.label, label_w);
            buf.set_string(area.x + 2, y, &label, text);
            crate::interact::record_legend(
                Rect::new(area.x, y, 2 + text_width(&label) as u16, 1),
                i,
                &ring.label,
            );
            let pct = format!("{:.0}%", ring.fraction() * 100.0);
            let px = area.right().saturating_sub(pct.len() as u16);
            if px > area.x + 2 + text_width(&label) as u16 {
                buf.set_string(
                    px,
                    y,
                    &pct,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                );
            }
        }
    }

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        let (chart, legend) = split(inner, &self.rings, self.legend);
        self.draw_rings(buf, chart);
        if let Some(l) = legend {
            self.draw_legend(buf, l);
        }
    }
}

widget_impls!(RadialBar);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for RadialBar {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

/// Splits `inner` into the rings area and an optional legend area.
fn split(inner: Rect, rings: &[RadialRing], legend: bool) -> (Rect, Option<Rect>) {
    if !legend || rings.is_empty() {
        return (inner, None);
    }
    let longest = rings
        .iter()
        .map(|r| text_width(&r.label))
        .max()
        .unwrap_or(0);
    let lw = (longest + 8).clamp(14, 32) as u16;
    // Keep at least a 12-column chart.
    if inner.width < lw + 12 {
        return (inner, None);
    }
    let chart_w = inner.width - lw - 1;
    let chart = Rect::new(inner.x, inner.y, chart_w, inner.height);
    let ly = inner.y + inner.height.saturating_sub(rings.len() as u16) / 2;
    let leg = Rect::new(
        inner.x + chart_w + 1,
        ly,
        lw,
        (rings.len() as u16).min(inner.bottom() - ly),
    );
    (chart, Some(leg))
}

/// Concentric 270° rings, one per `(label, percent)`, with a legend.
///
/// # Example
///
/// ```
/// use tui_chart::RingStack;
///
/// let stack = RingStack::new([("CPU", 72.0), ("RAM", 48.0), ("Disk", 91.0)]);
/// # let _ = stack;
/// ```
#[derive(Debug, Clone, Default)]
pub struct RingStack {
    items: Vec<(String, f64)>,
    colors: Vec<Color>,
    inner: RadialBar,
}

impl RingStack {
    /// Create a ring stack from `(label, percent)` pairs, outermost first.
    pub fn new<S: Into<String>>(items: impl IntoIterator<Item = (S, f64)>) -> Self {
        Self {
            items: items.into_iter().map(|(l, v)| (l.into(), v)).collect(),
            colors: Vec::new(),
            inner: RadialBar::new([]),
        }
    }

    /// Ring colors by position (the theme / palette fills in the rest).
    pub fn colors(mut self, colors: impl IntoIterator<Item = Color>) -> Self {
        self.colors = colors.into_iter().collect();
        self
    }

    /// Hide rings by index, see [`RadialBar::hidden`].
    #[must_use]
    pub fn hidden(mut self, hidden: impl IntoIterator<Item = usize>) -> Self {
        self.inner = self.inner.hidden(hidden);
        self
    }

    /// Show or hide the legend.
    pub fn legend(mut self, show: bool) -> Self {
        self.inner = self.inner.legend(show);
        self
    }

    /// Color of the unfilled track.
    pub fn track(mut self, c: Color) -> Self {
        self.inner = self.inner.track(c);
        self
    }

    /// Apply a [`Theme`]; explicit colors win.
    #[cfg(feature = "theme")]
    pub fn theme(mut self, theme: impl Into<Theme>) -> Self {
        self.inner = self.inner.theme(theme);
        self
    }

    /// Base style painted over the whole area.
    pub fn base_style(mut self, style: Style) -> Self {
        self.inner = self.inner.base_style(style);
        self
    }

    /// Wrap the chart in a [`Block`].
    pub fn block(mut self, b: Block<'static>) -> Self {
        self.inner = self.inner.block(b);
        self
    }

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let rings = self.items.iter().enumerate().map(|(i, (l, pct))| {
            let mut r = RadialRing::new(l.clone(), *pct, 100.0);
            r.color = self.colors.get(i).copied();
            r
        });
        let mut bar = self.inner.clone();
        bar.rings = rings.collect();
        bar.render_into(area, buf);
    }
}

widget_impls!(RingStack);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for RingStack {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::assert_is_empty, clippy::float_cmp)]
    use super::*;
    use ratatui::widgets::Widget;

    fn draw(w: impl Widget, width: u16, height: u16) -> Buffer {
        let a = Rect::new(0, 0, width, height);
        let mut b = Buffer::empty(a);
        w.render(a, &mut b);
        b
    }

    fn text(b: &Buffer) -> String {
        b.content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect()
    }

    fn rings() -> Vec<RadialRing> {
        vec![
            RadialRing::new("Move", 420.0, 600.0),
            RadialRing::new("Exercise", 25.0, 30.0).color(Color::Red),
            RadialRing::new("Stand", 9.0, 12.0),
        ]
    }

    #[test]
    fn fraction_guards() {
        assert_eq!(RadialRing::new("a", 5.0, 0.0).fraction(), 0.0);
        assert_eq!(RadialRing::new("a", f64::NAN, 10.0).fraction(), 0.0);
        assert_eq!(RadialRing::new("a", 50.0, 10.0).fraction(), 1.0);
        assert_eq!(RadialRing::new("a", -5.0, 10.0).fraction(), 0.0);
        assert!((RadialRing::new("a", 3.0, 12.0).fraction() - 0.25).abs() < 1e-12);
    }

    #[test]
    fn renders_rings_and_legend() {
        let b = draw(RadialBar::new(rings()), 50, 14);
        let t = text(&b);
        assert!(t.contains("Move") && t.contains("Exercise") && t.contains("70%"));
        assert!(b
            .content()
            .iter()
            .any(|c| c.fg == Color::Red && c.symbol() != " "));
    }

    #[test]
    fn legend_can_be_hidden() {
        let t = text(&draw(RadialBar::new(rings()).legend(false), 50, 14));
        assert!(!t.contains("Move"));
    }

    #[test]
    fn full_sweep_differs_from_arc() {
        let a = draw(RadialBar::new(rings()).legend(false), 30, 14);
        let b = draw(
            RadialBar::new(rings())
                .legend(false)
                .sweep(RadialSweep::Full),
            30,
            14,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn value_changes_picture() {
        let lo = draw(
            RadialBar::new(vec![RadialRing::new("a", 10.0, 100.0)]).legend(false),
            20,
            10,
        );
        let hi = draw(
            RadialBar::new(vec![RadialRing::new("a", 90.0, 100.0)]).legend(false),
            20,
            10,
        );
        assert_ne!(lo, hi);
    }

    #[test]
    fn robust_to_odd_sizes_and_input() {
        for (w, h) in [(0, 0), (1, 1), (4, 2), (13, 3), (60, 30)] {
            let _ = draw(RadialBar::new(rings()), w, h);
            let _ = draw(RadialBar::new(Vec::new()), w, h);
            let _ = draw(
                RadialBar::new(vec![RadialRing::new("x", f64::NAN, f64::NAN)]),
                w,
                h,
            );
            let many: Vec<RadialRing> = (0..40)
                .map(|i| RadialRing::new("r", f64::from(i), 40.0))
                .collect();
            let _ = draw(RadialBar::new(many), w, h);
        }
    }

    #[test]
    fn ring_stack_renders_percentages() {
        let t = text(&draw(
            RingStack::new([("CPU", 72.0), ("RAM", 48.0)]).block(Block::bordered().title("Sys")),
            50,
            14,
        ));
        assert!(t.contains("CPU") && t.contains("72%") && t.contains("Sys"));
    }

    #[test]
    fn ring_stack_custom_colors() {
        let b = draw(
            RingStack::new([("A", 100.0)])
                .colors([Color::Magenta])
                .legend(false),
            20,
            10,
        );
        assert!(b
            .content()
            .iter()
            .any(|c| c.fg == Color::Magenta && c.symbol() != " "));
    }
}
