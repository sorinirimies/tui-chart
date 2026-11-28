//! Voronoi chart: every point of the plot is painted with the color of its
//! nearest site, so the plot splits into one cell per site.
//!
//! Besides the classic diagram the widget covers a family of variants, all
//! driven by the same nearest-site engine:
//!
//! * **heatmap** – [`VoronoiSite::value`] + [`VoronoiChart::gradient`]
//! * **territory map / k-NN classifier** – [`VoronoiSite::category`],
//!   [`VoronoiChart::merge_categories`], [`VoronoiChart::knn`]
//! * **geometry** – [`VoronoiMetric`] (incl. Minkowski), [`VoronoiChart::farthest`],
//!   [`VoronoiChart::weighted`], [`VoronoiChart::relax`] (centroidal),
//!   [`VoronoiChart::delaunay`], [`VoronoiChart::shade_by_distance`]
//! * **interactive / animated** – [`VoronoiChart::phase`], [`VoronoiChart::probe`],
//!   [`VoronoiChart::select`]
//! * **scatter overlay** – [`VoronoiChart::axes`], [`VoronoiChart::bounds`],
//!   [`VoronoiChart::square`]
//! * **shapes & mosaics** – [`VoronoiChart::clip_circle`],
//!   [`voronoi_stipple`]
//!
//! For a Voronoi *treemap* (cell area ∝ value) see
//! [`VoronoiTreemap`](crate::VoronoiTreemap).

use crate::heatmap::Gradient;
use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::{
    darken, format_compact, format_tick, mix, nice_bounds, nice_ticks, pick, to_rgb,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};
use std::collections::BTreeMap;

/// Marks a pixel that belongs to no cell (clipped away).
const NONE: usize = usize::MAX;

/// A seed point of the diagram.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VoronoiSite {
    /// Site name, shown by [`VoronoiChart::show_labels`] and in tooltips.
    pub label: String,
    /// X position (data units).
    pub x: f64,
    /// Y position (data units).
    pub y: f64,
    /// Optional weight; with [`VoronoiChart::weighted`] heavier sites own larger cells.
    pub weight: Option<f64>,
    /// Optional measured value; with [`VoronoiChart::gradient`] it colors the cell.
    pub value: Option<f64>,
    /// Optional category; sites of one category share a color.
    pub category: Option<String>,
    /// Explicit color (beats value and category colors).
    pub color: Option<Color>,
}

impl VoronoiSite {
    /// Create a site at `(x, y)`.
    pub fn new(label: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            label: label.into(),
            x,
            y,
            weight: None,
            value: None,
            category: None,
            color: None,
        }
    }

    /// Weight (used by [`VoronoiChart::weighted`] and shown in the tooltip).
    pub fn weight(mut self, w: f64) -> Self {
        self.weight = Some(w);
        self
    }

    /// Measured value, mapped through [`VoronoiChart::gradient`].
    pub fn value(mut self, v: f64) -> Self {
        self.value = Some(v);
        self
    }

    /// Category; same-category sites share a color and (by default) merge borders.
    pub fn category(mut self, c: impl Into<String>) -> Self {
        self.category = Some(c.into());
        self
    }

    /// Explicit color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }

    fn finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

/// Distance used to decide which site is nearest.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VoronoiMetric {
    /// Straight-line distance: convex polygon cells (default).
    #[default]
    Euclidean,
    /// Taxicab distance `|dx| + |dy|`: diamond-flavoured cells.
    Manhattan,
    /// Chessboard distance `max(|dx|, |dy|)`: boxy cells.
    Chebyshev,
    /// Minkowski distance of order `p` (clamped to `0.5..=16`); `1` is
    /// Manhattan, `2` Euclidean, large `p` approaches Chebyshev.
    Minkowski(f64),
}

impl VoronoiMetric {
    /// The next metric, wrapping around (handy for a demo key).
    pub fn next(self) -> Self {
        match self {
            Self::Euclidean => Self::Manhattan,
            Self::Manhattan => Self::Chebyshev,
            Self::Chebyshev => Self::Minkowski(3.0),
            Self::Minkowski(_) => Self::Euclidean,
        }
    }

    fn dist(self, dx: f64, dy: f64) -> f64 {
        match self {
            Self::Euclidean => dx.hypot(dy),
            Self::Manhattan => dx + dy,
            Self::Chebyshev => dx.max(dy),
            Self::Minkowski(p) => {
                let p = if p.is_finite() {
                    p.clamp(0.5, 16.0)
                } else {
                    2.0
                };
                (dx.powf(p) + dy.powf(p)).powf(1.0 / p)
            }
        }
    }

    /// Monotone comparison key; with `r` it is the power-diagram key.
    fn key(self, dx: f64, dy: f64, r: f64) -> f64 {
        match self {
            Self::Euclidean => dx * dx + dy * dy - r * r,
            m => m.dist(dx, dy) - r,
        }
    }
}

/// Voronoi diagram widget.
///
/// Cells are drawn at half-block resolution (two rows of dots per terminal
/// row) in a screen-uniform space, so edges are the true perpendicular
/// bisectors as they appear on screen.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{Gradient, VoronoiChart, VoronoiSite};
///
/// let chart = VoronoiChart::new([
///     VoronoiSite::new("a", 1.0, 1.0).value(3.0),
///     VoronoiSite::new("b", 5.0, 3.0).value(9.0),
///     VoronoiSite::new("c", 3.0, 6.0).value(5.0),
/// ])
/// .gradient(Gradient::Viridis)
/// .show_labels(true);
/// let area = Rect::new(0, 0, 40, 14);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct VoronoiChart {
    sites: Vec<VoronoiSite>,
    metric: VoronoiMetric,
    weighted: bool,
    farthest: bool,
    borders: bool,
    show_sites: bool,
    show_labels: bool,
    fill_dim: f64,
    border_color: Option<Color>,
    gradient: Option<Gradient>,
    merge: bool,
    knn: usize,
    shade: bool,
    relax: usize,
    delaunay: bool,
    phase: f64,
    probe: Option<(f64, f64)>,
    select: Option<usize>,
    axes: bool,
    bounds: Option<(f64, f64, f64, f64)>,
    square: bool,
    clip_circle: bool,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

impl Default for VoronoiChart {
    fn default() -> Self {
        Self {
            sites: Vec::new(),
            metric: VoronoiMetric::default(),
            weighted: false,
            farthest: false,
            borders: true,
            show_sites: true,
            show_labels: false,
            fill_dim: 0.45,
            border_color: None,
            gradient: None,
            merge: true,
            knn: 1,
            shade: false,
            relax: 0,
            delaunay: false,
            phase: 0.0,
            probe: None,
            select: None,
            axes: false,
            bounds: None,
            square: false,
            clip_circle: false,
            theme: None,
            base: None,
            block: None,
        }
    }
}

/// One site as used while rendering (a real site or the probe).
struct Active {
    orig: Option<usize>,
    pos: (f64, f64),
    radius: f64,
}

/// Result of assigning every pixel to a site.
struct Field {
    own: Vec<usize>,
    dist: Vec<f64>,
}

impl VoronoiChart {
    /// Create a diagram from sites; sites with non-finite coordinates are skipped.
    pub fn new(sites: impl IntoIterator<Item = VoronoiSite>) -> Self {
        Self {
            sites: sites.into_iter().collect(),
            ..Self::default()
        }
    }

    setter!(
        /// Distance metric (default Euclidean).
        metric: VoronoiMetric
    );

    setter!(
        /// Weighted (power) diagram: heavier sites own larger cells (default off).
        weighted: bool
    );

    setter!(
        /// Farthest-site diagram: each point takes its *farthest* site (default off).
        farthest: bool
    );

    setter!(
        /// Draw lines along cell edges (default on).
        borders: bool
    );

    setter!(
        /// Mark each site with `●` (default on).
        show_sites: bool
    );

    setter!(
        /// Print the site label next to its marker (default off).
        show_labels: bool
    );

    setter!(
        /// How much cell fills are darkened toward black, `0.0..=1.0` (default 0.45).
        fill_dim: f64
    );

    opt_setter!(
        /// Edge color (default: a light tint of the cell color).
        border_color: Color
    );

    opt_setter!(
        /// Color cells by [`VoronoiSite::value`] through this gradient.
        gradient: Gradient
    );

    setter!(
        /// Merge borders between same-category cells (default on).
        merge_categories => merge: bool
    );

    setter!(
        /// k-nearest-neighbour classifier: each point takes the majority
        /// category of its `k` nearest sites (default 1 = plain Voronoi).
        knn: usize
    );

    setter!(
        /// Brightness follows the distance to the nearest site: glowing
        /// centres, dark edges (default off; replaces `fill_dim`).
        shade_by_distance => shade: bool
    );

    setter!(
        /// Lloyd relaxation steps: sites move to their cell centroids, giving
        /// an even, honeycomb-like (centroidal) diagram (default 0).
        relax: usize
    );

    setter!(
        /// Overlay the Delaunay graph: a line between every pair of
        /// neighbouring sites (default off).
        delaunay: bool
    );

    setter!(
        /// Animation phase; sites drift around their position (default 0).
        phase: f64
    );

    opt_setter!(
        /// Insert a temporary extra site at data position `(x, y)`, e.g. the
        /// mouse cursor, to show which part of the plot it would steal.
        probe: (f64, f64)
    );

    opt_setter!(
        /// Highlight the cell of site index `i` and outline its neighbours.
        select: usize
    );

    setter!(
        /// Draw x/y axes with tick labels, like a scatter plot (default off).
        axes: bool
    );

    /// Fix the visible data window instead of fitting it to the sites.
    pub fn bounds(mut self, x0: f64, x1: f64, y0: f64, y1: f64) -> Self {
        self.bounds = Some((x0, x1, y0, y1));
        self
    }

    setter!(
        /// Equal x/y scale (no stretching), e.g. for maps (default off).
        square: bool
    );

    setter!(
        /// Paint only the inscribed disc (default off).
        clip_circle: bool
    );

    theme_setter!(
        /// Apply a [`Theme`]; explicit site colors win.
        theme
    );

    base_style_setter!(base);

    block_setter!(block, 'static);

    fn assign(
        &self,
        act: &[Active],
        w: usize,
        h: usize,
        inside: &dyn Fn(usize, usize) -> bool,
    ) -> Field {
        let mut own = vec![NONE; w * h];
        let mut dist = vec![0.0; w * h];
        let m = self.metric;
        for py in 0..h {
            for px in 0..w {
                if !inside(px, py) {
                    continue;
                }
                let (qx, qy) = (px as f64 + 0.5, py as f64 + 0.5);
                let mut best = (if self.farthest { f64::MIN } else { f64::MAX }, 0usize, 0.0);
                for (k, a) in act.iter().enumerate() {
                    let (dx, dy) = ((qx - a.pos.0).abs(), (qy - a.pos.1).abs());
                    if self.farthest {
                        let d = m.dist(dx, dy);
                        if d > best.0 {
                            best = (d, k, d);
                        }
                    } else {
                        let key = m.key(dx, dy, a.radius);
                        if key < best.0 {
                            best = (key, k, 0.0);
                        }
                    }
                }
                let a = &act[best.1];
                own[py * w + px] = best.1;
                dist[py * w + px] = if self.farthest {
                    best.2
                } else {
                    m.dist((qx - a.pos.0).abs(), (qy - a.pos.1).abs())
                };
            }
        }
        Field { own, dist }
    }

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 2 || inner.height < 1 {
            return;
        }
        let valid: Vec<usize> = (0..self.sites.len())
            .filter(|&i| self.sites[i].finite())
            .collect();
        if valid.is_empty() {
            return;
        }
        // Data window.
        let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for &i in &valid {
            let s = &self.sites[i];
            x0 = x0.min(s.x);
            x1 = x1.max(s.x);
            y0 = y0.min(s.y);
            y1 = y1.max(s.y);
        }
        let ((x0, x1), (y0, y1)) = match self.bounds {
            Some((a, b, c, d)) if [a, b, c, d].iter().all(|v| v.is_finite()) && b > a && d > c => {
                ((a, b), (c, d))
            }
            _ if self.axes => (nice_bounds(x0, x1, 5), nice_bounds(y0, y1, 5)),
            _ => {
                let pad = |lo: f64, hi: f64| {
                    let span = hi - lo;
                    if span > f64::EPSILON {
                        (lo - span * 0.08, hi + span * 0.08)
                    } else {
                        (lo - 1.0, hi + 1.0)
                    }
                };
                (pad(x0, x1), pad(y0, y1))
            }
        };
        let (xs, ys) = ((x1 - x0).max(f64::EPSILON), (y1 - y0).max(f64::EPSILON));

        // Axes reserve a gutter and a label row.
        let (dim_style, text_style) = crate::util::dim_text(&self.theme);
        let yt = nice_ticks(y0, y1, (inner.height as usize / 3).max(2));
        let ystep = if yt.len() >= 2 { yt[1] - yt[0] } else { 1.0 };
        let ylabels: Vec<String> = yt.iter().map(|v| format_tick(*v, ystep)).collect();
        let gutter = if self.axes {
            (ylabels.iter().map(|l| l.chars().count()).max().unwrap_or(1) as u16 + 1)
                .min(inner.width / 2)
        } else {
            0
        };
        let plot = if self.axes {
            Rect::new(
                inner.x + gutter,
                inner.y,
                inner.width - gutter,
                inner.height.saturating_sub(1),
            )
        } else {
            inner
        };
        if plot.width < 2 || plot.height < 1 {
            return;
        }

        // Screen-uniform pixel grid: one dot per column, two per row.
        let (w, h) = (usize::from(plot.width), usize::from(plot.height) * 2);
        let (wf, hf) = (w as f64, h as f64);
        let (sx, sy) = if self.square {
            let s = (wf / xs).min(hf / ys);
            (s, s)
        } else {
            (wf / xs, hf / ys)
        };
        let (ox, oy) = ((wf - sx * xs) / 2.0, (hf - sy * ys) / 2.0);
        let to_px = |x: f64, y: f64| (ox + (x - x0) * sx, oy + (y1 - y) * sy);

        let inside = |px: usize, py: usize| -> bool {
            if !self.clip_circle {
                return true;
            }
            let (cx, cy, r) = (wf / 2.0, hf / 2.0, (wf / 2.0).min(hf / 2.0));
            let (dx, dy) = (px as f64 + 0.5 - cx, py as f64 + 0.5 - cy);
            dx * dx + dy * dy <= r * r
        };

        let spacing = ((w * h) as f64 / valid.len() as f64).sqrt();
        let wmax = valid
            .iter()
            .filter_map(|&i| self.sites[i].weight)
            .filter(|v| v.is_finite())
            .fold(0.0, f64::max);
        let mut act: Vec<Active> = valid
            .iter()
            .map(|&i| {
                let s = &self.sites[i];
                let radius = match s.weight {
                    Some(v) if self.weighted && v.is_finite() && wmax > 0.0 => {
                        (v.max(0.0) / wmax).sqrt() * 0.6 * spacing
                    }
                    _ => 0.0,
                };
                Active {
                    orig: Some(i),
                    pos: to_px(s.x, s.y),
                    radius,
                }
            })
            .collect();

        // Lloyd relaxation toward centroids.
        for _ in 0..self.relax.min(60) {
            let f = self.assign(&act, w, h, &inside);
            let mut sum = vec![(0.0, 0.0, 0usize); act.len()];
            for (p, &o) in f.own.iter().enumerate() {
                if o != NONE {
                    let e = &mut sum[o];
                    e.0 += (p % w) as f64 + 0.5;
                    e.1 += (p / w) as f64 + 0.5;
                    e.2 += 1;
                }
            }
            for (a, s) in act.iter_mut().zip(&sum) {
                if s.2 > 0 {
                    a.pos = (s.0 / s.2 as f64, s.1 / s.2 as f64);
                }
            }
        }
        // Drift.
        if self.phase != 0.0 && self.phase.is_finite() {
            for (k, a) in act.iter_mut().enumerate() {
                let (kf, amp) = (k as f64, 0.28 * spacing);
                a.pos.0 = (a.pos.0 + amp * (self.phase + kf * 1.7).sin()).clamp(0.0, wf - 1.0);
                a.pos.1 =
                    (a.pos.1 + amp * (self.phase * 0.8 + kf * 2.3).cos()).clamp(0.0, hf - 1.0);
            }
        }
        // Probe.
        let probe_idx = self
            .probe
            .filter(|(x, y)| x.is_finite() && y.is_finite())
            .map(|(x, y)| {
                let (px, py) = to_px(x, y);
                act.push(Active {
                    orig: None,
                    pos: (px.clamp(0.0, wf - 1.0), py.clamp(0.0, hf - 1.0)),
                    radius: 0.0,
                });
                act.len() - 1
            });

        let field = self.assign(&act, w, h, &inside);

        // ---- colors per active site ----
        let mut cats: Vec<&str> = Vec::new();
        let cat_of: Vec<Option<usize>> = act
            .iter()
            .map(|a| {
                let c = self.sites[a.orig?].category.as_deref()?;
                Some(cats.iter().position(|x| *x == c).unwrap_or_else(|| {
                    cats.push(c);
                    cats.len() - 1
                }))
            })
            .collect();
        let (mut vmin, mut vmax) = (f64::MAX, f64::MIN);
        for &i in &valid {
            if let Some(v) = self.sites[i].value.filter(|v| v.is_finite()) {
                vmin = vmin.min(v);
                vmax = vmax.max(v);
            }
        }
        let cat_color = |c: usize| -> Color {
            act.iter()
                .zip(&cat_of)
                .find(|(_, k)| **k == Some(c))
                .and_then(|(a, _)| a.orig)
                .and_then(|i| self.sites[i].color)
                .unwrap_or_else(|| pick(&self.theme, c))
        };
        let base: Vec<Color> = act
            .iter()
            .enumerate()
            .map(|(k, a)| {
                let Some(i) = a.orig else {
                    return Color::White;
                };
                let s = &self.sites[i];
                if let Some(c) = s.color {
                    c
                } else if let Some(c) = cat_of[k] {
                    cat_color(c)
                } else if let (Some(g), Some(v)) =
                    (&self.gradient, s.value.filter(|v| v.is_finite()))
                {
                    let t = if vmax > vmin {
                        (v - vmin) / (vmax - vmin)
                    } else {
                        0.5
                    };
                    g.color_at(t)
                } else if self.gradient.is_some() {
                    Color::DarkGray
                } else {
                    pick(&self.theme, i)
                }
            })
            .collect();

        // ---- per-pixel grouping and color ----
        let voting = self.knn >= 2 && !cats.is_empty();
        let n = w * h;
        let mut grp = vec![NONE; n];
        let mut fill_base: Vec<Option<Color>> = vec![None; n];
        for p in 0..n {
            let o = field.own[p];
            if o == NONE {
                continue;
            }
            if voting {
                let (qx, qy) = ((p % w) as f64 + 0.5, (p / w) as f64 + 0.5);
                let mut ds: Vec<(f64, usize)> = act
                    .iter()
                    .enumerate()
                    .filter(|(k, _)| cat_of[*k].is_some())
                    .map(|(k, a)| {
                        (
                            self.metric.dist((qx - a.pos.0).abs(), (qy - a.pos.1).abs()),
                            k,
                        )
                    })
                    .collect();
                ds.sort_by(|a, b| a.0.total_cmp(&b.0));
                let mut votes = vec![0usize; cats.len()];
                for (_, k) in ds.iter().take(self.knn) {
                    if let Some(c) = cat_of[*k] {
                        votes[c] += 1;
                    }
                }
                let top = *votes.iter().max().unwrap_or(&0);
                // ties go to the category of the nearest site
                let win = ds
                    .iter()
                    .filter_map(|(_, k)| cat_of[*k])
                    .find(|c| votes[*c] == top)
                    .unwrap_or(0);
                grp[p] = win;
                fill_base[p] = Some(cat_color(win));
            } else {
                grp[p] = match (self.merge, cat_of[o]) {
                    (true, Some(c)) => c,
                    _ => cats.len() + o,
                };
                fill_base[p] = Some(base[o]);
            }
        }

        let dmax = (spacing * 0.8).max(1.0);
        let dim = self.fill_dim.clamp(0.0, 1.0);
        let mut px: Vec<Option<Color>> = (0..n)
            .map(|p| {
                let b = fill_base[p]?;
                Some(if self.shade {
                    let f = (field.dist[p] / dmax).clamp(0.0, 1.0);
                    darken(mix(b, Color::White, 0.15 * (1.0 - f)), f * 0.85)
                } else {
                    darken(b, dim)
                })
            })
            .collect();

        // ---- adjacency (needed by delaunay, selection and tooltips) ----
        let mut adj: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for p in 0..n {
            let a = field.own[p];
            if a == NONE {
                continue;
            }
            let right = (p % w + 1 < w).then(|| field.own[p + 1]);
            let down = (p / w + 1 < h).then(|| field.own[p + w]);
            for b in [right, down].into_iter().flatten() {
                if b != NONE && b != a {
                    *adj.entry((a.min(b), a.max(b))).or_default() += 1;
                }
            }
        }
        let sel = self
            .select
            .and_then(|i| act.iter().position(|a| a.orig == Some(i)));

        // ---- highlight selection ----
        if let Some(s) = sel {
            for p in 0..n {
                if field.own[p] == s {
                    px[p] = px[p].map(|c| mix(c, Color::White, 0.28));
                }
            }
        }

        // ---- edges ----
        if self.borders {
            let edge_of = |p: usize| -> Color {
                self.border_color.unwrap_or_else(|| {
                    mix(fill_base[p].unwrap_or(Color::White), Color::White, 0.55)
                })
            };
            let mut paint = Vec::new();
            for p in 0..n {
                if grp[p] == NONE {
                    continue;
                }
                let (x, y) = (p % w, p / w);
                let mut differs = false;
                let mut near_sel = false;
                for q in [
                    (x > 0).then(|| p - 1),
                    (x + 1 < w).then(|| p + 1),
                    (y > 0).then(|| p - w),
                    (y + 1 < h).then(|| p + w),
                ]
                .into_iter()
                .flatten()
                {
                    if grp[q] == NONE || grp[q] == grp[p] {
                        continue;
                    }
                    let sel_edge = sel.is_some_and(|s| field.own[q] == s || field.own[p] == s);
                    // one dot wide: the lower-id side owns the line
                    if grp[q] > grp[p] || sel_edge {
                        differs = true;
                        if let Some(s) = sel
                            && (field.own[q] == s || field.own[p] == s)
                        {
                            near_sel = true;
                        }
                    }
                }
                if differs {
                    paint.push((p, if near_sel { Color::White } else { edge_of(p) }));
                }
            }
            for (p, c) in paint {
                px[p] = Some(c);
            }
        }

        // ---- delaunay overlay ----
        if self.delaunay {
            for &(a, b) in adj.keys() {
                let (p, q) = (act[a].pos, act[b].pos);
                let steps = (q.0 - p.0).abs().max((q.1 - p.1).abs()).ceil().max(1.0) as usize;
                for t in 0..=steps {
                    let f = t as f64 / steps as f64;
                    let (x, y) = (p.0 + (q.0 - p.0) * f, p.1 + (q.1 - p.1) * f);
                    if x >= 0.0 && y >= 0.0 && (x as usize) < w && (y as usize) < h {
                        let i = y as usize * w + x as usize;
                        if let Some(c) = px[i] {
                            px[i] = Some(mix(c, Color::White, 0.75));
                        }
                    }
                }
            }
        }

        paint_pixels(buf, plot, &px, w);

        // ---- markers and labels ----
        let at = |a: &Active| -> (u16, u16) {
            (
                plot.x + (a.pos.0.max(0.0) as usize).min(w - 1) as u16,
                plot.y + ((a.pos.1.max(0.0) as usize) / 2).min(usize::from(plot.height) - 1) as u16,
            )
        };
        for (k, a) in act.iter().enumerate() {
            let (x, y) = at(a);
            let bg = buf[(x, y)].bg;
            let bg = if bg == Color::Reset { base[k] } else { bg };
            let ink = contrast(bg);
            if self.show_sites || Some(k) == probe_idx {
                let glyph = if Some(k) == probe_idx { '◆' } else { '●' };
                buf[(x, y)]
                    .set_char(glyph)
                    .set_style(Style::default().fg(ink).bg(bg));
            }
            if self.show_labels
                && let Some(i) = a.orig
                && !self.sites[i].label.is_empty()
            {
                let lx = if self.show_sites { x + 2 } else { x };
                put_str(
                    buf,
                    lx,
                    y,
                    plot.right(),
                    &self.sites[i].label,
                    Style::default().fg(ink).bg(bg),
                );
            }
        }

        // ---- axes ----
        if self.axes {
            for (v, l) in yt.iter().zip(&ylabels) {
                let (_, py) = to_px(x0, *v);
                let y = plot.y + ((py / 2.0).max(0.0) as u16).min(plot.height - 1);
                let lw = l.chars().count() as u16;
                put_str(
                    buf,
                    inner.x + gutter.saturating_sub(lw + 1),
                    y,
                    plot.x,
                    l,
                    dim_style,
                );
            }
            let xt = nice_ticks(x0, x1, (plot.width as usize / 10).max(2));
            let xstep = if xt.len() >= 2 { xt[1] - xt[0] } else { 1.0 };
            for v in xt {
                let (px, _) = to_px(v, y0);
                let x = plot.x + (px.max(0.0) as u16).min(plot.width - 1);
                let l = format_tick(v, xstep);
                let lx = x.saturating_sub(l.chars().count() as u16 / 2).max(plot.x);
                put_str(buf, lx, inner.bottom() - 1, inner.right(), &l, text_style);
            }
        }

        // ---- hit regions ----
        if crate::interact::recording() {
            let owners: Vec<Option<usize>> = (0..usize::from(plot.height))
                .flat_map(|cy| (0..w).map(move |cx| (cx, cy)))
                .map(|(cx, cy)| {
                    let o = field.own[cy * 2 * w + cx];
                    (o != NONE && act[o].orig.is_some()).then_some(o)
                })
                .collect();
            let inside_total = field.own.iter().filter(|&&o| o != NONE).count().max(1);
            crate::interact::record_grid(
                plot,
                &owners,
                |k| act[k].orig.unwrap_or(0),
                |k| {
                    let s = &self.sites[act[k].orig.unwrap_or(0)];
                    let mut lines = vec![format!(
                        "x {}  y {}",
                        format_compact(s.x),
                        format_compact(s.y)
                    )];
                    if let Some(v) = s.weight {
                        lines.push(format!("weight {}", format_compact(v)));
                    }
                    if let Some(v) = s.value {
                        lines.push(format!("value {}", format_compact(v)));
                    }
                    if let Some(c) = &s.category {
                        lines.push(format!("category {c}"));
                    }
                    let owned = field.own.iter().filter(|&&o| o == k).count();
                    lines.push(format!(
                        "area {:.0}%",
                        owned as f64 / inside_total as f64 * 100.0
                    ));
                    // neighbours and the share of the border each one takes
                    let mut nb: Vec<(usize, usize)> = adj
                        .iter()
                        .filter_map(|(&(a, b), &c)| {
                            (a == k).then_some((b, c)).or((b == k).then_some((a, c)))
                        })
                        .collect();
                    let perim: usize = nb.iter().map(|x| x.1).sum::<usize>().max(1);
                    nb.sort_by_key(|x| std::cmp::Reverse(x.1));
                    if !nb.is_empty() {
                        let list: Vec<String> = nb
                            .iter()
                            .take(3)
                            .map(|(o, c)| {
                                let name = act[*o]
                                    .orig
                                    .map_or("probe", |i| self.sites[i].label.as_str());
                                format!("{name} {}%", c * 100 / perim)
                            })
                            .collect();
                        lines.push(format!("{} neighbours: {}", nb.len(), list.join(", ")));
                    }
                    (s.label.clone(), lines)
                },
            );
        }
    }
}

/// Paint a `w`-wide dot grid (two dots per cell row) into `plot` with half
/// blocks. `None` pixels leave the cell untouched.
pub(crate) fn paint_pixels(buf: &mut Buffer, plot: Rect, px: &[Option<Color>], w: usize) {
    for cy in 0..usize::from(plot.height) {
        for cx in 0..w {
            let (top, bot) = (px[cy * 2 * w + cx], px[(cy * 2 + 1) * w + cx]);
            let cell = &mut buf[(plot.x + cx as u16, plot.y + cy as u16)];
            match (top, bot) {
                (None, None) => {}
                (Some(t), Some(b)) if t == b => {
                    cell.set_char(' ').set_style(Style::default().bg(t));
                }
                (Some(t), Some(b)) => {
                    cell.set_char('▀').set_style(Style::default().fg(t).bg(b));
                }
                (Some(t), None) => {
                    cell.set_char('▀')
                        .set_style(Style::default().fg(t).bg(Color::Reset));
                }
                (None, Some(b)) => {
                    cell.set_char('▄')
                        .set_style(Style::default().fg(b).bg(Color::Reset));
                }
            }
        }
    }
}

/// Black or white, whichever reads better on `bg`.
pub(crate) fn contrast(bg: Color) -> Color {
    let (r, g, b) = to_rgb(bg);
    if 0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b) > 140.0 {
        Color::Black
    } else {
        Color::White
    }
}

/// Stipple / mosaic generator: sample `n` sites from a density grid
/// (`density[row][col]`, row 0 on top) and relax them `relax` times toward the
/// density-weighted centroids, so dots crowd where the density is high.
///
/// Each site carries the density at its position as [`VoronoiSite::value`], so
/// `VoronoiChart::new(voronoi_stipple(..)).gradient(..)` gives a mosaic.
/// Deterministic for a given `seed`; non-finite or negative densities count as 0.
///
/// ```
/// use tui_chart::voronoi_stipple;
/// let density = vec![vec![0.0, 1.0, 0.0], vec![1.0, 5.0, 1.0]];
/// let sites = voronoi_stipple(&density, 8, 3, 1);
/// assert_eq!(sites.len(), 8);
/// ```
pub fn voronoi_stipple(
    density: &[Vec<f64>],
    n: usize,
    relax: usize,
    seed: u64,
) -> Vec<VoronoiSite> {
    let rows = density.len();
    let cols = density.iter().map(Vec::len).max().unwrap_or(0);
    let d = |r: usize, c: usize| {
        density
            .get(r)
            .and_then(|row| row.get(c))
            .copied()
            .filter(|v| v.is_finite() && *v > 0.0)
            .unwrap_or(0.0)
    };
    let total: f64 = (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .map(|(r, c)| d(r, c))
        .sum();
    if rows == 0 || cols == 0 || n == 0 || total <= 0.0 {
        return Vec::new();
    }
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let mut rnd = move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    // inverse-CDF sampling over cells, uniform inside the cell
    let mut cdf = Vec::with_capacity(rows * cols);
    let mut acc = 0.0;
    for r in 0..rows {
        for c in 0..cols {
            acc += d(r, c);
            cdf.push(acc);
        }
    }
    let mut pos: Vec<(f64, f64)> = (0..n)
        .map(|_| {
            let t = rnd() * total;
            let i = cdf.partition_point(|v| *v < t).min(cdf.len() - 1);
            ((i % cols) as f64 + rnd(), (i / cols) as f64 + rnd())
        })
        .collect();
    for _ in 0..relax.min(60) {
        let mut sum = vec![(0.0, 0.0, 0.0); n];
        for r in 0..rows {
            for c in 0..cols {
                let wgt = d(r, c);
                if wgt == 0.0 {
                    continue;
                }
                let (qx, qy) = (c as f64 + 0.5, r as f64 + 0.5);
                let k = pos
                    .iter()
                    .enumerate()
                    .min_by(|a, b| {
                        let da = (a.1.0 - qx).powi(2) + (a.1.1 - qy).powi(2);
                        let db = (b.1.0 - qx).powi(2) + (b.1.1 - qy).powi(2);
                        da.total_cmp(&db)
                    })
                    .map_or(0, |x| x.0);
                sum[k].0 += qx * wgt;
                sum[k].1 += qy * wgt;
                sum[k].2 += wgt;
            }
        }
        for (p, s) in pos.iter_mut().zip(&sum) {
            if s.2 > 0.0 {
                *p = (s.0 / s.2, s.1 / s.2);
            }
        }
    }
    pos.into_iter()
        .enumerate()
        .map(|(i, (x, y))| {
            let (c, r) = ((x as usize).min(cols - 1), (y as usize).min(rows - 1));
            VoronoiSite::new(format!("s{i}"), x, rows as f64 - y).value(d(r, c))
        })
        .collect()
}

widget_impls!(VoronoiChart);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for VoronoiChart {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    const RED: Color = Color::Rgb(200, 0, 0);
    const BLUE: Color = Color::Rgb(0, 0, 200);

    fn render(c: &VoronoiChart, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    fn count_bg(b: &Buffer, c: Color) -> usize {
        b.content().iter().filter(|x| x.bg == c).count()
    }

    fn two(a: f64, b: f64) -> [VoronoiSite; 2] {
        [
            VoronoiSite::new("a", 0.0, 0.0).color(RED).weight(a),
            VoronoiSite::new("b", 10.0, 0.0).color(BLUE).weight(b),
        ]
    }

    fn flat(c: VoronoiChart) -> VoronoiChart {
        c.borders(false).fill_dim(0.0).show_sites(false)
    }

    #[test]
    fn two_sites_split_plot_in_half() {
        let b = render(&flat(VoronoiChart::new(two(1.0, 1.0))), 40, 10);
        let (l, r) = (count_bg(&b, RED), count_bg(&b, BLUE));
        assert!(l > 100 && r > 100 && l.abs_diff(r) <= 20, "{l} vs {r}");
    }

    #[test]
    fn heavier_site_owns_more_when_weighted() {
        let area = |wt: bool| {
            let c = flat(VoronoiChart::new(two(100.0, 1.0)).weighted(wt));
            count_bg(&render(&c, 60, 12), RED)
        };
        assert!(area(true) > area(false));
    }

    #[test]
    fn every_metric_fills_the_plot() {
        for m in [
            VoronoiMetric::Euclidean,
            VoronoiMetric::Manhattan,
            VoronoiMetric::Chebyshev,
            VoronoiMetric::Minkowski(3.0),
            VoronoiMetric::Minkowski(f64::NAN),
        ] {
            let c = VoronoiChart::new(two(1.0, 1.0)).metric(m).show_labels(true);
            let b = render(&c, 30, 8);
            assert!(b.content().iter().all(|x| x.bg != Color::Reset), "{m:?}");
        }
    }

    #[test]
    fn metric_next_cycles_back() {
        let mut m = VoronoiMetric::Euclidean;
        for _ in 0..4 {
            m = m.next();
        }
        assert_eq!(m, VoronoiMetric::Euclidean);
    }

    #[test]
    fn minkowski_two_matches_euclidean() {
        let a = render(&VoronoiChart::new(two(1.0, 1.0)), 30, 8);
        let b = render(
            &VoronoiChart::new(two(1.0, 1.0)).metric(VoronoiMetric::Minkowski(2.0)),
            30,
            8,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn labels_are_drawn() {
        let b = render(&VoronoiChart::new(two(1.0, 1.0)).show_labels(true), 40, 10);
        assert!(b.content().iter().any(|x| x.symbol() == "a"));
        assert!(b.content().iter().any(|x| x.symbol() == "●"));
    }

    #[test]
    fn gradient_colors_cells_by_value() {
        let c = flat(VoronoiChart::new([
            VoronoiSite::new("lo", 0.0, 0.0).value(0.0),
            VoronoiSite::new("hi", 10.0, 0.0).value(10.0),
        ]))
        .gradient(Gradient::Grayscale);
        let b = render(&c, 40, 10);
        assert!(count_bg(&b, Color::Rgb(0, 0, 0)) > 50);
        assert!(count_bg(&b, Color::Rgb(255, 255, 255)) > 50);
    }

    #[test]
    fn categories_share_color_and_merge_borders() {
        let sites = || [0.0, 4.0, 8.0].map(|x| VoronoiSite::new("s", x, 0.0).category("same"));
        let edges = |merge: bool| {
            let c = VoronoiChart::new(sites())
                .merge_categories(merge)
                .show_sites(false)
                .fill_dim(0.0);
            let b = render(&c, 40, 10);
            // distinct bg colors: just the fill when merged, fill + edge otherwise
            let mut set: Vec<Color> = b.content().iter().map(|x| x.bg).collect();
            set.sort_by_key(|c| format!("{c:?}"));
            set.dedup();
            set.len()
        };
        assert!(edges(false) > edges(true));
    }

    #[test]
    fn knn_votes_by_majority() {
        // one lone "b" amid two "a": with k=3 the whole plot becomes "a"
        let sites = [
            VoronoiSite::new("a1", 0.0, 0.0).category("a").color(RED),
            VoronoiSite::new("a2", 1.0, 0.0).category("a").color(RED),
            VoronoiSite::new("b1", 10.0, 0.0).category("b").color(BLUE),
        ];
        let c = |k| flat(VoronoiChart::new(sites.clone()).knn(k));
        let b1 = render(&c(1), 40, 8);
        let b3 = render(&c(3), 40, 8);
        assert!(count_bg(&b1, BLUE) > 0);
        assert_eq!(count_bg(&b3, BLUE), 0);
    }

    #[test]
    fn farthest_flips_ownership() {
        let sites = || two(1.0, 1.0);
        let at_left = |far: bool| {
            let b = render(&flat(VoronoiChart::new(sites()).farthest(far)), 40, 6);
            b[(1, 2)].bg
        };
        assert_eq!(at_left(false), RED);
        assert_eq!(at_left(true), BLUE);
    }

    #[test]
    fn shade_by_distance_varies_within_a_cell() {
        let c = VoronoiChart::new([VoronoiSite::new("a", 0.0, 0.0).color(Color::Rgb(200, 200, 0))])
            .shade_by_distance(true)
            .borders(false)
            .show_sites(false);
        let b = render(&c, 40, 10);
        let mut set: Vec<Color> = b.content().iter().map(|x| x.bg).collect();
        set.sort_by_key(|c| format!("{c:?}"));
        set.dedup();
        assert!(set.len() > 5);
    }

    #[test]
    fn relax_evens_out_cells() {
        let sites = [(0.0, 0.0), (1.0, 0.5), (2.0, 0.0), (10.0, 10.0)].map(|(x, y)| (x, y));
        let build = |n| {
            VoronoiChart::new(
                sites
                    .iter()
                    .enumerate()
                    .map(|(i, &(x, y))| VoronoiSite::new(format!("{i}"), x, y)),
            )
            .relax(n)
        };
        let a = render(&build(0), 50, 14);
        let b = render(&build(12), 50, 14);
        assert_ne!(a, b);
    }

    #[test]
    fn delaunay_overlay_changes_output() {
        let c = VoronoiChart::new(two(1.0, 1.0));
        assert_ne!(
            render(&c, 40, 10),
            render(&c.clone().delaunay(true), 40, 10)
        );
    }

    #[test]
    fn phase_moves_sites() {
        let c = VoronoiChart::new(two(1.0, 1.0));
        assert_ne!(render(&c, 40, 10), render(&c.clone().phase(1.3), 40, 10));
    }

    #[test]
    fn probe_steals_area_and_draws_marker() {
        let c = flat(VoronoiChart::new(two(1.0, 1.0)));
        let before = count_bg(&render(&c, 40, 10), RED);
        let with = render(&c.clone().probe((2.0, 0.0)), 40, 10);
        assert!(count_bg(&with, RED) < before);
        assert!(with.content().iter().any(|x| x.symbol() == "◆"));
    }

    #[test]
    fn select_highlights_cell() {
        let c = flat(VoronoiChart::new(two(1.0, 1.0)));
        assert_ne!(render(&c, 40, 10), render(&c.clone().select(0), 40, 10));
    }

    #[test]
    fn axes_draw_tick_labels() {
        let c = VoronoiChart::new(two(1.0, 1.0)).axes(true);
        let b = render(&c, 50, 12);
        let last: String = (0..50).map(|x| b[(x, 11)].symbol().to_string()).collect();
        assert!(last.contains('5') || last.contains('0'), "{last}");
    }

    #[test]
    fn square_axes_ticks_follow_the_centred_plot() {
        // wide plot, square scale: data fills only the middle, so the x
        // tick for the left bound must sit right of the plot's left edge
        let c = VoronoiChart::new(two(1.0, 1.0))
            .bounds(0.0, 10.0, 0.0, 10.0)
            .square(true)
            .axes(true);
        let b = render(&c, 80, 12);
        let row: String = (0..80).map(|x| b[(x, 11)].symbol().to_string()).collect();
        let first = row.find('0').unwrap();
        assert!(first > 20, "{row:?}");
    }

    #[test]
    fn square_keeps_aspect_and_bounds_are_honoured() {
        let c = flat(VoronoiChart::new(two(1.0, 1.0)))
            .bounds(-5.0, 15.0, -5.0, 5.0)
            .square(true);
        render(&c, 50, 14);
        // invalid bounds fall back to fitting
        render(&c.clone().bounds(f64::NAN, 1.0, 0.0, 1.0), 50, 14);
        render(&c.bounds(3.0, 1.0, 0.0, 1.0), 50, 14);
    }

    #[test]
    fn clip_circle_leaves_corners_untouched() {
        let c = flat(VoronoiChart::new(two(1.0, 1.0))).clip_circle(true);
        let b = render(&c, 40, 20);
        assert_eq!(b[(0, 0)].bg, Color::Reset);
        assert_ne!(b[(20, 10)].bg, Color::Reset);
    }

    #[test]
    fn stipple_is_deterministic_and_follows_density() {
        let mut d = vec![vec![0.0; 20]; 10];
        for row in d.iter_mut() {
            for v in row.iter_mut().skip(10) {
                *v = 1.0;
            }
        }
        let a = voronoi_stipple(&d, 30, 4, 9);
        assert_eq!(a, voronoi_stipple(&d, 30, 4, 9));
        assert_eq!(a.len(), 30);
        assert!(a.iter().all(|s| s.x >= 10.0), "dots stay in the dense half");
        assert!(voronoi_stipple(&[], 5, 1, 1).is_empty());
        assert!(voronoi_stipple(&[vec![f64::NAN, -1.0]], 5, 1, 1).is_empty());
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let one = VoronoiChart::new([VoronoiSite::new("a", 1.0, 1.0)])
            .weighted(true)
            .relax(3)
            .delaunay(true)
            .axes(true)
            .knn(5)
            .select(0);
        let bad = VoronoiChart::new([
            VoronoiSite::new("n", f64::NAN, 1.0),
            VoronoiSite::new("a", 2.0, 2.0)
                .weight(f64::NAN)
                .value(f64::NAN),
            VoronoiSite::new("b", 2.0, 2.0).weight(-3.0).category("x"),
        ])
        .weighted(true)
        .show_labels(true)
        .gradient(Gradient::Heat)
        .knn(2)
        .phase(f64::NAN)
        .probe((f64::NAN, 1.0))
        .select(99);
        let all = VoronoiChart::new(two(1.0, 2.0))
            .farthest(true)
            .clip_circle(true)
            .axes(true)
            .probe((100.0, -100.0))
            .shade_by_distance(true);
        for c in [&one, &bad, &all, &VoronoiChart::new([])] {
            render(c, 40, 12);
            render(c, 1, 1);
            render(c, 3, 2);
            render(c, 0, 0);
        }
    }
}
