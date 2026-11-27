//! Voronoi treemap: like [`Treemap`](crate::Treemap) but cells are organic
//! Voronoi regions whose area is proportional to the item value. Groups
//! (items with children) are partitioned recursively inside their own region.
//!
//! The partition is a weighted (power) diagram whose sites are moved to their
//! cell centroids and whose weights are adjusted until every cell has its
//! target area (a few dozen iterations on the dot grid, deterministic for a
//! given [`VoronoiTreemap::seed`]).

use crate::theme::{self, Theme};
use crate::treemap::TreemapItem;
use crate::util::{darken, format_compact, mix, pick};
use crate::voronoi::{contrast, paint_pixels};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

const NONE: usize = usize::MAX;

/// Voronoi treemap widget. Reuses [`TreemapItem`] for its data.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{TreemapItem, VoronoiTreemap};
///
/// let chart = VoronoiTreemap::new([
///     TreemapItem::new("web", 0.0).children([
///         TreemapItem::new("api", 40.0),
///         TreemapItem::new("ui", 25.0),
///     ]),
///     TreemapItem::new("infra", 30.0),
/// ]);
/// let area = Rect::new(0, 0, 50, 16);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct VoronoiTreemap {
    items: Vec<TreemapItem>,
    seed: u64,
    labels: bool,
    borders: bool,
    iterations: usize,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

/// A leaf of the flattened hierarchy.
struct Leaf<'a> {
    item: &'a TreemapItem,
    top: usize,
    path: String,
    weight: f64,
}

impl VoronoiTreemap {
    /// Create a treemap from (possibly nested) items; zero/invalid weights get no cell.
    pub fn new(items: impl IntoIterator<Item = TreemapItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            seed: 7,
            labels: true,
            borders: true,
            iterations: 40,
            theme: None,
            base: None,
            block: None,
        }
    }

    setter!(
        /// Seed of the initial site placement (default 7); same seed, same picture.
        seed: u64
    );

    setter!(
        /// Print leaf labels (default on).
        labels: bool
    );

    setter!(
        /// Draw cell borders; group borders are brighter (default on).
        borders: bool
    );

    setter!(
        /// Relaxation iterations per level, upper bound (default 40). Large
        /// areas lower it automatically to keep rendering fast.
        iterations: usize
    );

    theme_setter!(
        /// Apply a [`Theme`]; explicit item colors win.
        theme
    );

    base_style_setter!(base);

    block_setter!(block, 'static);

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 2 || inner.height < 1 {
            return;
        }
        let mut leaves: Vec<Leaf> = Vec::new();
        let mut tops: Vec<&TreemapItem> = Vec::new();
        for it in &self.items {
            if it.weight() > 0.0 {
                tops.push(it);
            }
        }
        if tops.is_empty() {
            return;
        }
        for (ti, it) in tops.iter().enumerate() {
            Self::flatten(it, ti, String::new(), &mut leaves);
        }
        let (w, h) = (usize::from(inner.width), usize::from(inner.height) * 2);
        let mut leaf_of = vec![NONE; w * h];
        let mut top_of = vec![NONE; w * h];

        // Top level.
        let all: Vec<usize> = (0..w * h).collect();
        let top_w: Vec<f64> = tops.iter().map(|t| t.weight()).collect();
        let top_assign = partition(&all, w, &top_w, self.seed, self.iterations);
        let mut regions: Vec<Vec<usize>> = vec![Vec::new(); tops.len()];
        for (p, a) in all.iter().zip(&top_assign) {
            regions[*a].push(*p);
            top_of[*p] = *a;
        }
        // Inside each top-level item.
        for (ti, region) in regions.iter().enumerate() {
            let mine: Vec<usize> = leaves
                .iter()
                .enumerate()
                .filter(|(_, l)| l.top == ti)
                .map(|(i, _)| i)
                .collect();
            self.fill_group(tops[ti], region, w, &mine, &mut leaf_of, ti as u64);
        }

        // Colors.
        let leaf_color = |li: usize| -> Color {
            let l = &leaves[li];
            if let Some(c) = l.item.color {
                return c;
            }
            let base = tops[l.top]
                .color
                .unwrap_or_else(|| pick(&self.theme, l.top));
            let in_top: Vec<usize> = leaves
                .iter()
                .enumerate()
                .filter(|(_, x)| x.top == l.top)
                .map(|(i, _)| i)
                .collect();
            if in_top.len() <= 1 {
                return base;
            }
            let rank = in_top.iter().position(|&i| i == li).unwrap_or(0) as f64;
            let t = rank / (in_top.len() - 1) as f64;
            mix(darken(base, 0.25), Color::White, 0.12 + 0.28 * t)
        };
        let colors: Vec<Color> = (0..leaves.len()).map(leaf_color).collect();
        let mut px: Vec<Option<Color>> = (0..w * h)
            .map(|p| (leaf_of[p] != NONE).then(|| colors[leaf_of[p]]))
            .collect();

        if self.borders {
            let mut paint = Vec::new();
            for p in 0..w * h {
                if leaf_of[p] == NONE {
                    continue;
                }
                let (x, y) = (p % w, p / w);
                let (mut leaf_edge, mut top_edge) = (false, false);
                for q in [
                    (x > 0).then(|| p - 1),
                    (x + 1 < w).then(|| p + 1),
                    (y > 0).then(|| p - w),
                    (y + 1 < h).then(|| p + w),
                ]
                .into_iter()
                .flatten()
                {
                    if leaf_of[q] != NONE && leaf_of[q] > leaf_of[p] {
                        leaf_edge = true;
                        top_edge |= top_of[q] != top_of[p];
                    }
                }
                if top_edge {
                    paint.push((p, mix(colors[leaf_of[p]], Color::White, 0.75)));
                } else if leaf_edge {
                    paint.push((p, darken(colors[leaf_of[p]], 0.5)));
                }
            }
            for (p, c) in paint {
                px[p] = Some(c);
            }
        }
        paint_pixels(buf, inner, &px, w);

        // Labels at the centroid, clipped to the leaf's horizontal run.
        let mut sum = vec![(0usize, 0usize, 0usize); leaves.len()];
        for (p, &l) in leaf_of.iter().enumerate() {
            if l != NONE {
                sum[l].0 += p % w;
                sum[l].1 += p / w;
                sum[l].2 += 1;
            }
        }
        if self.labels {
            for (li, s) in sum.iter().enumerate() {
                if s.2 == 0 {
                    continue;
                }
                let (cx, cy) = (s.0 / s.2, (s.1 / s.2) / 2);
                let at = |x: usize| leaf_of[cy * 2 * w + x];
                if at(cx) != li {
                    continue;
                }
                let (mut a, mut b) = (cx, cx);
                while a > 0 && at(a - 1) == li {
                    a -= 1;
                }
                while b + 1 < w && at(b + 1) == li {
                    b += 1;
                }
                let room = b - a + 1;
                let label: String = leaves[li].item.label.chars().take(room).collect();
                if room < 3 || label.is_empty() {
                    continue;
                }
                let lx = a + (room - label.chars().count()) / 2;
                let ink = contrast(colors[li]);
                for (i, ch) in label.chars().enumerate() {
                    let cell = &mut buf[(inner.x + (lx + i) as u16, inner.y + cy as u16)];
                    let bg = if cell.bg == Color::Reset {
                        colors[li]
                    } else {
                        cell.bg
                    };
                    cell.set_char(ch).set_style(Style::default().fg(ink).bg(bg));
                }
            }
        }

        if crate::interact::recording() {
            let owners: Vec<Option<usize>> = (0..usize::from(inner.height))
                .flat_map(|cy| (0..w).map(move |cx| (cx, cy)))
                .map(|(cx, cy)| {
                    let l = leaf_of[cy * 2 * w + cx];
                    (l != NONE).then_some(l)
                })
                .collect();
            let total: f64 = leaves.iter().map(|l| l.weight).sum();
            crate::interact::record_grid(
                inner,
                &owners,
                |l| leaves[l].top,
                |l| {
                    let lf = &leaves[l];
                    let share = if total > 0.0 {
                        lf.weight / total * 100.0
                    } else {
                        0.0
                    };
                    let area = sum[l].2 as f64 / (w * h) as f64 * 100.0;
                    (
                        lf.path.clone(),
                        vec![
                            format!("value {}", format_compact(lf.weight)),
                            format!("share {share:.0}%  area {area:.0}%"),
                        ],
                    )
                },
            );
        }
    }

    fn flatten<'a>(it: &'a TreemapItem, top: usize, path: String, out: &mut Vec<Leaf<'a>>) {
        let path = if path.is_empty() {
            it.label.clone()
        } else {
            format!("{path} › {}", it.label)
        };
        if it.children.is_empty() {
            out.push(Leaf {
                item: it,
                top,
                path,
                weight: it.weight(),
            });
        } else {
            for c in it.children.iter().filter(|c| c.weight() > 0.0) {
                Self::flatten(c, top, path.clone(), out);
            }
        }
    }

    /// Partition `region` among the children of `item` (recursively) and
    /// write the winning leaf index per pixel.
    #[allow(clippy::too_many_arguments)]
    fn fill_group(
        &self,
        item: &TreemapItem,
        region: &[usize],
        w: usize,
        mine: &[usize],
        leaf_of: &mut [usize],
        salt: u64,
    ) {
        if item.children.is_empty() {
            if let Some(&l) = mine.first() {
                for &p in region {
                    leaf_of[p] = l;
                }
            }
            return;
        }
        let kids: Vec<&TreemapItem> = item.children.iter().filter(|c| c.weight() > 0.0).collect();
        if kids.is_empty() || region.is_empty() {
            return;
        }
        let weights: Vec<f64> = kids.iter().map(|k| k.weight()).collect();
        let assign = partition(
            region,
            w,
            &weights,
            self.seed.wrapping_add(salt.wrapping_mul(0x9E37_79B9)),
            self.iterations,
        );
        // leaves are stored in flatten order, so each kid owns a contiguous run
        let mut offset = 0;
        for (ki, kid) in kids.iter().enumerate() {
            let count = Self::leaf_count(kid);
            let sub: Vec<usize> = region
                .iter()
                .zip(&assign)
                .filter(|(_, a)| **a == ki)
                .map(|(p, _)| *p)
                .collect();
            let slice = &mine[offset..offset + count];
            self.fill_group(kid, &sub, w, slice, leaf_of, salt * 31 + ki as u64 + 1);
            offset += count;
        }
    }

    fn leaf_count(it: &TreemapItem) -> usize {
        if it.children.is_empty() {
            1
        } else {
            it.children
                .iter()
                .filter(|c| c.weight() > 0.0)
                .map(Self::leaf_count)
                .sum()
        }
    }
}

/// Split the pixels of `region` (indices into a `w`-wide grid) into
/// `weights.len()` power-diagram cells with areas proportional to `weights`.
/// Returns the cell index of every entry of `region`.
fn partition(
    region: &[usize],
    w: usize,
    weights: &[f64],
    seed: u64,
    max_iter: usize,
) -> Vec<usize> {
    let k = weights.len();
    if k <= 1 || region.is_empty() {
        return vec![0; region.len()];
    }
    let total: f64 = weights.iter().sum();
    let target: Vec<f64> = weights
        .iter()
        .map(|v| v / total * region.len() as f64)
        .collect();
    let pts: Vec<(f64, f64)> = region
        .iter()
        .map(|&p| ((p % w) as f64 + 0.5, (p / w) as f64 + 0.5))
        .collect();

    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let mut rnd = move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (
            (state >> 33) as usize,
            (state >> 11) as f64 / (1u64 << 53) as f64,
        )
    };
    // spread-out start: best of a few random candidates per site
    let mut pos: Vec<(f64, f64)> = Vec::with_capacity(k);
    for _ in 0..k {
        let mut best = (-1.0, pts[0]);
        for _ in 0..8 {
            let c = pts[rnd().0 % pts.len()];
            let d = pos
                .iter()
                .map(|q: &(f64, f64)| (q.0 - c.0).powi(2) + (q.1 - c.1).powi(2))
                .fold(f64::MAX, f64::min);
            if d > best.0 {
                best = (d, c);
            }
        }
        pos.push(best.1);
    }
    let spacing2 = region.len() as f64 / k as f64;
    let iters = (2_000_000 / (region.len() * k).max(1)).clamp(10, max_iter.max(1));
    let mut wgt = vec![0.0f64; k];
    let mut assign = vec![0usize; region.len()];
    for it in 0..=iters {
        let mut cnt = vec![0usize; k];
        let mut cen = vec![(0.0, 0.0); k];
        for (i, &(x, y)) in pts.iter().enumerate() {
            let mut best = (f64::MAX, 0);
            for (c, p) in pos.iter().enumerate() {
                let d = (x - p.0).powi(2) + (y - p.1).powi(2) - wgt[c];
                if d < best.0 {
                    best = (d, c);
                }
            }
            assign[i] = best.1;
            cnt[best.1] += 1;
            cen[best.1].0 += x;
            cen[best.1].1 += y;
        }
        if it == iters {
            break;
        }
        let step = 0.6 * (1.0 - it as f64 / iters as f64) + 0.1;
        for c in 0..k {
            if cnt[c] > 0 {
                let g = (cen[c].0 / cnt[c] as f64, cen[c].1 / cnt[c] as f64);
                pos[c].0 += (g.0 - pos[c].0) * 0.5;
                pos[c].1 += (g.1 - pos[c].1) * 0.5;
            } else {
                pos[c] = pts[rnd().0 % pts.len()];
            }
            let err = (target[c] - cnt[c] as f64) / target[c].max(1.0);
            wgt[c] += step * err.clamp(-1.0, 1.0) * spacing2 * 0.5;
        }
    }
    assign
}

widget_impls!(VoronoiTreemap);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for VoronoiTreemap {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(c: &VoronoiTreemap, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    fn items() -> Vec<TreemapItem> {
        vec![
            TreemapItem::new("big", 60.0).color(Color::Rgb(200, 0, 0)),
            TreemapItem::new("mid", 30.0).color(Color::Rgb(0, 200, 0)),
            TreemapItem::new("small", 10.0).color(Color::Rgb(0, 0, 200)),
        ]
    }

    fn share(b: &Buffer, c: Color) -> f64 {
        let all = b.content().iter().filter(|x| x.bg != Color::Reset).count();
        b.content().iter().filter(|x| x.bg == c).count() as f64 / all.max(1) as f64
    }

    #[test]
    fn areas_follow_values() {
        let b = render(
            &VoronoiTreemap::new(items()).borders(false).labels(false),
            80,
            30,
        );
        let (a, m, s) = (
            share(&b, Color::Rgb(200, 0, 0)),
            share(&b, Color::Rgb(0, 200, 0)),
            share(&b, Color::Rgb(0, 0, 200)),
        );
        assert!((a - 0.6).abs() < 0.12, "big {a}");
        assert!((m - 0.3).abs() < 0.12, "mid {m}");
        assert!((s - 0.1).abs() < 0.08, "small {s}");
        assert!(a > m && m > s);
    }

    #[test]
    fn fills_the_whole_area() {
        let b = render(&VoronoiTreemap::new(items()), 60, 20);
        assert!(b.content().iter().all(|x| x.bg != Color::Reset));
    }

    #[test]
    fn deterministic_and_seed_changes_layout() {
        let t = VoronoiTreemap::new(items());
        assert_eq!(render(&t, 60, 20), render(&t, 60, 20));
        assert_ne!(render(&t, 60, 20), render(&t.clone().seed(99), 60, 20));
    }

    #[test]
    fn groups_partition_recursively() {
        let t = VoronoiTreemap::new([
            TreemapItem::new("g1", 0.0)
                .color(Color::Rgb(200, 0, 0))
                .children([TreemapItem::new("a", 30.0), TreemapItem::new("b", 10.0)]),
            TreemapItem::new("g2", 0.0)
                .color(Color::Rgb(0, 0, 200))
                .children([TreemapItem::new("c", 40.0), TreemapItem::new("d", 20.0)]),
        ])
        .labels(false);
        let b = render(&t, 80, 30);
        let mut distinct: Vec<String> = b.content().iter().map(|x| format!("{:?}", x.bg)).collect();
        distinct.sort();
        distinct.dedup();
        // four leaf shades plus borders
        assert!(distinct.len() >= 4, "{distinct:?}");
    }

    #[test]
    fn labels_are_printed() {
        let b = render(&VoronoiTreemap::new(items()), 80, 30);
        let text: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("big"), "no label");
        let b = render(&VoronoiTreemap::new(items()).labels(false), 80, 30);
        let text: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(!text.contains("big"));
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let nasty = VoronoiTreemap::new([
            TreemapItem::new("n", f64::NAN),
            TreemapItem::new("z", 0.0),
            TreemapItem::new("i", f64::INFINITY),
            TreemapItem::new("neg", -4.0),
            TreemapItem::new("g", 0.0).children([TreemapItem::new("e", 0.0)]),
            TreemapItem::new("one", 1.0),
        ]);
        let single = VoronoiTreemap::new([TreemapItem::new("a", 3.0)]);
        for t in [&nasty, &single, &VoronoiTreemap::new([])] {
            render(t, 40, 12);
            render(t, 1, 1);
            render(t, 3, 2);
            render(t, 0, 0);
            render(t, 300, 80);
        }
    }
}
