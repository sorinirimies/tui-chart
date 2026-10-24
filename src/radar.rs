//! Radar (spider) chart drawn on a braille canvas.

use crate::canvas::{Canvas, Marker};
use crate::theme::{self, Theme};
use crate::util::darken;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// One data series of a [`RadarChart`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RadarSeries {
    /// Series name.
    pub name: String,
    /// One value per axis.
    pub values: Vec<f64>,
    /// Optional series color (overrides the theme palette).
    pub color: Option<Color>,
}

impl RadarSeries {
    /// Create a series.
    pub fn new(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            name: name.into(),
            values: values.into_iter().collect(),
            color: None,
        }
    }
    opt_setter!(
        /// Set color.
        color: Color
    );
}

/// Spider chart with N axes and any number of series.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{RadarChart, RadarSeries};
///
/// let chart = RadarChart::new(["Speed", "Power", "Armor"]).series(RadarSeries::new("A", [3.0, 7.0, 5.0]));
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct RadarChart<'a> {
    axes: Vec<String>,
    series: Vec<RadarSeries>,
    max: Option<f64>,
    rings: usize,
    fill: bool,
    show_labels: bool,
    legend: bool,
    hidden: Vec<usize>,
    theme: Option<Theme>,
    base: Option<Style>,
    marker: Marker,
    block: Option<Block<'a>>,
}

impl<'a> RadarChart<'a> {
    /// Create a chart from axis labels.
    pub fn new(axes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            axes: axes.into_iter().map(Into::into).collect(),
            series: Vec::new(),
            max: None,
            rings: 4,
            fill: false,
            show_labels: true,
            legend: true,
            hidden: vec![],
            theme: None,
            base: None,
            marker: Marker::Braille,
            block: None,
        }
    }
    /// Add a series.
    pub fn series(mut self, s: RadarSeries) -> Self {
        self.series.push(s);
        self
    }
    opt_setter!(
        /// Fix the outer-ring value (default: largest data value).
        max: f64
    );
    setter!(
        /// Number of concentric rings.
        rings: usize
    );
    setter!(
        /// Fill the polygons instead of outlining.
        fill: bool
    );
    setter!(
        /// Show axis labels.
        show_labels: bool
    );
    theme_setter!(
        /// Apply a [`Theme`]: palette series colors, grid/label colors and
        /// background fill. Explicit [`RadarSeries::color`] wins.
        theme
    );
    base_style_setter!(base);
    fn series_color(&self, i: usize, s: &RadarSeries) -> Color {
        s.color
            .unwrap_or_else(|| self.theme.as_ref().map_or(Color::Cyan, |t| t.color(i)))
    }
    setter!(
        /// Glyph family used for dot drawing (default [`Marker::Braille`]).
        marker: Marker
    );

    setter!(
        /// Series-name legend on the bottom row (default on, shown when it fits).
        legend: bool
    );

    /// Hide series by index; they stay in the legend so they can be toggled
    /// back, see [`Interaction`](crate::Interaction).
    #[must_use]
    pub fn hidden(mut self, hidden: impl IntoIterator<Item = usize>) -> Self {
        self.hidden = hidden.into_iter().collect();
        self
    }

    block_setter!(block, 'a);

    fn draw(&self, area: Rect, buf: &mut Buffer) {
        let n = self.axes.len();
        if n < 3 || area.width < 3 || area.height < 3 {
            return;
        }
        let legend = self.legend && area.height >= 9 && !self.series.is_empty();
        let full = area;
        let area = if legend {
            Rect::new(area.x, area.y, area.width, area.height - 1)
        } else {
            area
        };
        let lab_w = self
            .axes
            .iter()
            .map(|a| a.chars().count())
            .max()
            .unwrap_or(0) as u16;
        let labels = self.show_labels && area.height >= 5 && area.width > 2 * lab_w + 4;
        let (mx, my) = if labels { (lab_w, 1) } else { (0, 0) };
        let plot = Rect::new(
            area.x + mx,
            area.y + my,
            area.width - 2 * mx,
            area.height - 2 * my,
        );
        let max = self
            .max
            .filter(|m| m.is_finite() && *m > 0.0)
            .unwrap_or_else(|| {
                let m = self
                    .series
                    .iter()
                    .flat_map(|s| s.values.iter().copied())
                    .filter(|v| v.is_finite())
                    .fold(0.0, f64::max);
                if m > 0.0 {
                    m
                } else {
                    1.0
                }
            });
        let mut cv = Canvas::new(plot.width, plot.height, self.marker);
        let (pw, ph) = (cv.px_w() as f64, cv.px_h() as f64);
        let r = ((pw - 1.0) / 2.0).min((ph - 1.0) / 2.0).max(1.0);
        let (cx, cy) = ((pw - 1.0) / 2.0, (ph - 1.0) / 2.0);
        let ang =
            |i: usize| -std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU * i as f64 / n as f64;
        let pt = |i: usize, rad: f64| {
            (
                (cx + rad * ang(i).cos()).round() as i32,
                (cy + rad * ang(i).sin()).round() as i32,
            )
        };
        let grid = self.theme.as_ref().map_or(Color::DarkGray, |t| t.grid);
        let fg = self.theme.as_ref().map_or(Color::White, |t| t.foreground);
        for k in 1..=self.rings {
            let rad = r * k as f64 / self.rings as f64;
            for i in 0..n {
                let (a, b) = (pt(i, rad), pt((i + 1) % n, rad));
                cv.line(a.0, a.1, b.0, b.1, grid);
            }
        }
        for i in 0..n {
            let b = pt(i, r);
            cv.line(cx.round() as i32, cy.round() as i32, b.0, b.1, grid);
        }
        let mut polys: Vec<(Color, Vec<(i32, i32)>)> = vec![];
        for (si, s) in self.series.iter().enumerate() {
            if self.hidden.contains(&si) {
                continue;
            }
            let color = self.series_color(si, s);
            let pts: Vec<(i32, i32)> = (0..n)
                .map(|i| {
                    let v = s
                        .values
                        .get(i)
                        .copied()
                        .filter(|v| v.is_finite())
                        .unwrap_or(0.0);
                    pt(i, r * (v / max).clamp(0.0, 1.0))
                })
                .collect();
            if crate::interact::recording() {
                let (rx, ry) = self.marker.resolution();
                for (i, &(x, y)) in pts.iter().enumerate() {
                    let rect = Rect::new(
                        plot.x + (x.max(0) / i32::from(rx)) as u16,
                        plot.y + (y.max(0) / i32::from(ry)) as u16,
                        1,
                        1,
                    )
                    .intersection(plot);
                    crate::interact::record_point(rect, si, i, || {
                        let v = s.values.get(i).copied().unwrap_or(0.0);
                        (
                            s.name.clone(),
                            vec![format!(
                                "{}: {}",
                                self.axes[i],
                                crate::util::format_compact(v)
                            )],
                        )
                    });
                }
            }
            polys.push((color, pts));
        }
        if self.fill {
            // scanline even-odd fill in a dimmed series color
            for (color, pts) in &polys {
                let dim = darken(*color, 0.55);
                for py in 0..ph as i32 {
                    let mut xs: Vec<f64> = vec![];
                    for i in 0..n {
                        let (a, b) = (pts[i], pts[(i + 1) % n]);
                        let (y0, y1) = (a.1.min(b.1), a.1.max(b.1));
                        if py >= y0 && py < y1 {
                            let t = (py - a.1) as f64 / (b.1 - a.1) as f64;
                            xs.push(a.0 as f64 + t * (b.0 - a.0) as f64);
                        }
                    }
                    xs.sort_by(|a, b| a.total_cmp(b));
                    for pair in xs.chunks(2) {
                        if let [x0, x1] = pair {
                            for px in x0.round() as i32..=x1.round() as i32 {
                                cv.set(px, py, dim);
                            }
                        }
                    }
                }
            }
        }
        for (color, pts) in &polys {
            for i in 0..n {
                let (a, b) = (pts[i], pts[(i + 1) % n]);
                cv.line(a.0, a.1, b.0, b.1, *color);
            }
            for &(x, y) in pts {
                for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                    cv.set(x + dx, y + dy, *color);
                }
            }
        }
        cv.render(buf, plot);
        // ring values along the first axis (up), dim
        if self.rings > 0 && plot.height >= 9 {
            let (rxd, ryd) = self.marker.resolution();
            let dim = Style::default().fg(darken(fg, 0.4));
            let ccx = plot.x as i32 + plot.width as i32 / 2;
            let ccy = plot.y as i32 + plot.height as i32 / 2;
            for k in 1..=self.rings {
                let v = max * k as f64 / self.rings as f64;
                let y = ccy - (r * k as f64 / self.rings as f64 / ryd as f64).round() as i32;
                let _ = rxd;
                let t = crate::util::format_tick(v, max / self.rings as f64);
                if y > plot.y as i32 && y < plot.bottom() as i32 && ccx + 2 < plot.right() as i32 {
                    crate::text::put_str(buf, (ccx + 2) as u16, y as u16, plot.right(), &t, dim);
                }
            }
        }
        if legend {
            let mut x = full.x + 1;
            let y = full.bottom() - 1;
            for (si, se) in self.series.iter().enumerate() {
                let c = self.series_color(si, se);
                let name = format!(" {}", se.name);
                if x + 2 + name.chars().count() as u16 > full.right() {
                    break;
                }
                let off = self.hidden.contains(&si);
                let name_style = if off {
                    Style::default().fg(fg).add_modifier(
                        ratatui::style::Modifier::CROSSED_OUT | ratatui::style::Modifier::DIM,
                    )
                } else {
                    Style::default().fg(fg)
                };
                crate::text::put_str(
                    buf,
                    x,
                    y,
                    full.right(),
                    if off { "○" } else { "●" },
                    Style::default().fg(c),
                );
                crate::text::put_str(buf, x + 1, y, full.right(), &name, name_style);
                crate::interact::record_legend(
                    Rect::new(x, y, 2 + name.chars().count() as u16, 1),
                    si,
                    &se.name,
                );
                x += 3 + name.chars().count() as u16;
            }
        }
        if labels {
            for (i, l) in self.axes.iter().enumerate() {
                let (ux, uy) = (ang(i).cos(), ang(i).sin());
                let w = l.chars().count() as i32;
                let ccx = plot.x as i32 + plot.width as i32 / 2;
                let ccy = plot.y as i32 + plot.height as i32 / 2;
                let (rxd, ryd) = self.marker.resolution();
                let (rx, ry) = (r / rxd as f64, r / ryd as f64);
                let ex = ccx + (ux * (rx + 1.5)).round() as i32;
                let ey = ccy + (uy * (ry + 1.0)).round() as i32;
                let x = if ux > 0.3 {
                    ex
                } else if ux < -0.3 {
                    ex - w
                } else {
                    ex - w / 2
                };
                let y = ey.clamp(area.y as i32, area.bottom() as i32 - 1);
                for (j, ch) in l.chars().enumerate() {
                    let xx = x + j as i32;
                    if xx >= area.x as i32 && xx < area.right() as i32 {
                        if let Some(c) = buf.cell_mut((xx as u16, y as u16)) {
                            c.set_char(ch).set_fg(fg);
                        }
                    }
                }
            }
        }
    }
}

impl RadarChart<'_> {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        self.draw(inner, buf);
    }
}

widget_impls!(RadarChart<'_>);

#[cfg(feature = "theme")]
impl<'a> tui_nuances::Themed for RadarChart<'a> {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn chart() -> RadarChart<'static> {
        RadarChart::new(["Atk", "Def", "Spd", "Int", "Hp"])
            .series(RadarSeries::new("a", [5.0, 3.0, 4.0, 2.0, 5.0]).color(Color::Red))
            .max(5.0)
    }

    fn run(c: RadarChart, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    #[test]
    fn draws_series_and_labels() {
        let b = run(chart(), 40, 20);
        assert!(b
            .content()
            .iter()
            .any(|c| c.fg == Color::Red && c.symbol() != " "));
        assert!(b
            .content()
            .iter()
            .any(|c| c.fg == Color::DarkGray && c.symbol() != " "));
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains("Atk") && t.contains("Hp"));
    }

    #[cfg(feature = "theme")]
    #[test]
    fn theme_palette_and_override() {
        let t = Theme::nord();
        let mk = |c: Option<Color>| {
            let s = RadarSeries::new("a", [5.0, 3.0, 4.0, 2.0, 5.0]);
            let s = if let Some(c) = c { s.color(c) } else { s };
            RadarChart::new(["Atk", "Def", "Spd", "Int", "Hp"])
                .series(s)
                .series(RadarSeries::new("b", [1.0, 2.0, 3.0, 4.0, 5.0]))
                .max(5.0)
                .theme(&t)
        };
        let has = |b: &Buffer, c: Color| b.content().iter().any(|x| x.fg == c && x.symbol() != " ");
        let b = run(mk(None), 40, 20);
        assert!(has(&b, t.color(0)) && has(&b, t.color(1)));
        assert_eq!(b[(0, 0)].bg, t.background.unwrap());
        let b = run(mk(Some(Color::Red)), 40, 20);
        assert!(has(&b, Color::Red) && !has(&b, t.color(0)) && has(&b, t.color(1)));
    }

    #[test]
    fn fill_covers_more_cells() {
        let count = |b: &Buffer| {
            b.content()
                .iter()
                .filter(|c| c.symbol() != " " && c.fg != Color::DarkGray && c.fg != Color::White)
                .count()
        };
        assert!(count(&run(chart().fill(true), 40, 20)) > count(&run(chart(), 40, 20)));
    }

    #[test]
    fn degenerate_inputs() {
        run(chart(), 0, 0);
        run(chart(), 3, 3);
        run(chart(), 5, 40);
        run(RadarChart::new(["a"]), 20, 10);
        run(
            RadarChart::new(["a", "b", "c"])
                .series(RadarSeries::new("n", [f64::NAN, -1.0]))
                .rings(0),
            20,
            10,
        );
    }
}
