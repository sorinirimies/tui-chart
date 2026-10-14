//! Radar (spider) chart drawn on a braille canvas.

use crate::canvas::{Canvas, Marker};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{Block, Widget},
};

/// One data series of a [`RadarChart`].
#[derive(Debug, Clone)]
pub struct RadarSeries {
    /// Series name.
    pub name: String,
    /// One value per axis.
    pub values: Vec<f64>,
    /// Series color.
    pub color: Color,
}

impl RadarSeries {
    /// Create a series.
    pub fn new(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            name: name.into(),
            values: values.into_iter().collect(),
            color: Color::Cyan,
        }
    }
    /// Set color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }
}

/// Spider chart with N axes and any number of series.
#[derive(Debug, Clone)]
pub struct RadarChart<'a> {
    axes: Vec<String>,
    series: Vec<RadarSeries>,
    max: Option<f64>,
    rings: usize,
    fill: bool,
    show_labels: bool,
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
            block: None,
        }
    }
    /// Add a series.
    pub fn series(mut self, s: RadarSeries) -> Self {
        self.series.push(s);
        self
    }
    /// Fix the outer-ring value (default: largest data value).
    pub fn max(mut self, m: f64) -> Self {
        self.max = Some(m);
        self
    }
    /// Number of concentric rings.
    pub fn rings(mut self, n: usize) -> Self {
        self.rings = n;
        self
    }
    /// Fill the polygons instead of outlining.
    pub fn fill(mut self, f: bool) -> Self {
        self.fill = f;
        self
    }
    /// Show axis labels.
    pub fn show_labels(mut self, s: bool) -> Self {
        self.show_labels = s;
        self
    }
    /// Wrap in a block.
    pub fn block(mut self, b: Block<'a>) -> Self {
        self.block = Some(b);
        self
    }

    fn draw(&self, area: Rect, buf: &mut Buffer) {
        let n = self.axes.len();
        if n < 3 || area.width < 3 || area.height < 3 {
            return;
        }
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
        let mut cv = Canvas::new(plot.width, plot.height, Marker::Braille);
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
        let grid = Color::DarkGray;
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
        for s in &self.series {
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
            if self.fill {
                for i in 0..n {
                    let (a, b) = (pts[i], pts[(i + 1) % n]);
                    let steps = (a.0 - b.0).abs().max((a.1 - b.1).abs()).max(1);
                    for t in 0..=steps {
                        let x = a.0 + (b.0 - a.0) * t / steps;
                        let y = a.1 + (b.1 - a.1) * t / steps;
                        cv.line(cx.round() as i32, cy.round() as i32, x, y, s.color);
                    }
                }
            }
            for i in 0..n {
                let (a, b) = (pts[i], pts[(i + 1) % n]);
                cv.line(a.0, a.1, b.0, b.1, s.color);
            }
        }
        cv.render(buf, plot);
        if labels {
            for (i, l) in self.axes.iter().enumerate() {
                let (ux, uy) = (ang(i).cos(), ang(i).sin());
                let w = l.chars().count() as i32;
                let ccx = plot.x as i32 + plot.width as i32 / 2;
                let ccy = plot.y as i32 + plot.height as i32 / 2;
                let ex = ccx + (ux * (plot.width as f64 / 2.0)).round() as i32;
                let ey = ccy + (uy * (plot.height as f64 / 2.0 + 1.0)).round() as i32;
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
                            c.set_char(ch).set_fg(Color::White);
                        }
                    }
                }
            }
        }
    }
}

impl Widget for RadarChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl Widget for &RadarChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match &self.block {
            Some(b) => {
                let i = b.inner(area);
                b.clone().render(area, buf);
                i
            }
            None => area,
        };
        self.draw(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn fill_covers_more_cells() {
        let count = |b: &Buffer| b.content().iter().filter(|c| c.fg == Color::Red).count();
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
