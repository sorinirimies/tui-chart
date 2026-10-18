//! Sankey diagram: nodes in columns joined by flow ribbons whose thickness
//! encodes the value.

use crate::canvas::{Canvas, Marker};
use crate::text::put_str;
use crate::theme::{self, Theme};
use crate::util::{darken, dim_text, format_compact, pick};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Block,
};

/// A flow from one node to another.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SankeyLink {
    /// Source node name.
    pub from: String,
    /// Target node name.
    pub to: String,
    /// Flow amount (non-positive and non-finite links are ignored).
    pub value: f64,
}

impl SankeyLink {
    /// Create a link.
    pub fn new(from: impl Into<String>, to: impl Into<String>, value: f64) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            value,
        }
    }
}

/// Sankey diagram widget. Nodes are created from the link endpoints and laid
/// out left to right by their longest path from a source.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{Sankey, SankeyLink};
///
/// let chart = Sankey::new([SankeyLink::new("In", "Mid", 10.0), SankeyLink::new("Mid", "Out", 7.0)]);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct Sankey {
    links: Vec<SankeyLink>,
    colors: Vec<(String, Color)>,
    node_width: u16,
    node_gap: u16,
    show_values: bool,
    marker: Marker,
    theme: Option<Theme>,
    base: Option<Style>,
    block: Option<Block<'static>>,
}

struct Node {
    name: String,
    layer: usize,
    inflow: f64,
    outflow: f64,
    y: f64,
    h: f64,
}

impl Sankey {
    /// Create a diagram.
    pub fn new(links: impl IntoIterator<Item = SankeyLink>) -> Self {
        Self {
            links: links.into_iter().collect(),
            colors: Vec::new(),
            node_width: 2,
            node_gap: 1,
            show_values: true,
            marker: Marker::HalfBlock,
            theme: None,
            base: None,
            block: None,
        }
    }

    /// Color a node (and its outgoing ribbons) by name.
    pub fn node_color(mut self, name: impl Into<String>, color: Color) -> Self {
        self.colors.push((name.into(), color));
        self
    }

    setter!(
        /// Node bar width in cells (default 2).
        node_width: u16
    );
    setter!(
        /// Empty rows between nodes of a column (default 1).
        node_gap: u16
    );
    setter!(
        /// Append the flow amount to node labels (default on).
        show_values: bool
    );
    setter!(
        /// Dot resolution (default half blocks, which fill solid).
        marker: Marker
    );
    theme_setter!(
        /// Apply a [`Theme`]; explicit node colors win.
        theme
    );
    base_style_setter!(base);
    block_setter!(block, 'static);

    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 12 || inner.height < 3 {
            return;
        }
        // ── graph ────────────────────────────────────────────────────────
        let mut nodes: Vec<Node> = Vec::new();
        let index = |name: &str, nodes: &mut Vec<Node>| -> usize {
            if let Some(i) = nodes.iter().position(|n| n.name == name) {
                return i;
            }
            nodes.push(Node {
                name: name.to_string(),
                layer: 0,
                inflow: 0.0,
                outflow: 0.0,
                y: 0.0,
                h: 0.0,
            });
            nodes.len() - 1
        };
        let mut links: Vec<(usize, usize, f64)> = Vec::new();
        for l in &self.links {
            if !(l.value.is_finite() && l.value > 0.0) || l.from == l.to {
                continue;
            }
            let (a, b) = (index(&l.from, &mut nodes), index(&l.to, &mut nodes));
            nodes[a].outflow += l.value;
            nodes[b].inflow += l.value;
            links.push((a, b, l.value));
        }
        let n = nodes.len();
        if n < 2 {
            return;
        }
        // longest-path layering; bounded passes keep cycles finite
        for _ in 0..n {
            let mut changed = false;
            for &(a, b, _) in &links {
                if nodes[b].layer < nodes[a].layer + 1 && nodes[a].layer < n {
                    nodes[b].layer = nodes[a].layer + 1;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let mut used: Vec<usize> = nodes.iter().map(|n| n.layer).collect();
        used.sort_unstable();
        used.dedup();
        for nd in nodes.iter_mut() {
            nd.layer = used.iter().position(|l| *l == nd.layer).unwrap_or(0);
        }
        let layers = used.len();
        if layers < 2 {
            return;
        }
        let flow = |nd: &Node| nd.inflow.max(nd.outflow);
        let label_of = |nd: &Node| {
            if self.show_values {
                format!("{} {}", nd.name, format_compact(flow(nd)))
            } else {
                nd.name.clone()
            }
        };
        let (dim, text) = dim_text(&self.theme);

        // ── geometry ─────────────────────────────────────────────────────
        let cap = inner.width / 4;
        let left_w = nodes
            .iter()
            .filter(|n| n.layer == 0)
            .map(|n| label_of(n).chars().count() as u16 + 1)
            .max()
            .unwrap_or(0)
            .min(cap);
        let right_w = nodes
            .iter()
            .filter(|n| n.layer == layers - 1)
            .map(|n| label_of(n).chars().count() as u16 + 1)
            .max()
            .unwrap_or(0)
            .min(cap);
        let plot = Rect::new(
            inner.x + left_w,
            inner.y,
            inner.width.saturating_sub(left_w + right_w),
            inner.height,
        );
        if plot.width < 2 * self.node_width + 2 || plot.height < 2 {
            return;
        }
        let mut cv = Canvas::new(plot.width, plot.height, self.marker);
        let (pw, ph) = (cv.px_w(), cv.px_h());
        let (rxd, ryd) = self.marker.resolution();
        let nw = (self.node_width.max(1) as i32) * rxd as i32;
        let gap = self.node_gap as i32 * ryd as i32;
        // scale so the fullest layer fits
        let mut k = f64::MAX;
        for l in 0..layers {
            let col: Vec<&Node> = nodes.iter().filter(|n| n.layer == l).collect();
            let total: f64 = col.iter().map(|n| flow(n)).sum();
            let avail = ph as f64 - (gap * (col.len() as i32 - 1)) as f64;
            if total > 0.0 {
                k = k.min(avail.max(col.len() as f64) / total);
            }
        }
        if !k.is_finite() {
            return;
        }
        for l in 0..layers {
            let ids: Vec<usize> = (0..n).filter(|i| nodes[*i].layer == l).collect();
            let used_h: f64 = ids.iter().map(|i| flow(&nodes[*i]) * k).sum::<f64>()
                + (gap * (ids.len() as i32 - 1)) as f64;
            let mut y = (ph as f64 - used_h) / 2.0;
            for i in ids {
                nodes[i].y = y;
                nodes[i].h = (flow(&nodes[i]) * k).max(1.0);
                y += nodes[i].h + gap as f64;
            }
        }
        let col_x = |layer: usize| -> i32 {
            if layers == 1 {
                0
            } else {
                (layer as i32 * (pw - nw)) / (layers as i32 - 1)
            }
        };
        let color_of = |i: usize| -> Color {
            self.colors
                .iter()
                .rev()
                .find(|(name, _)| *name == nodes[i].name)
                .map_or_else(|| pick(&self.theme, i), |(_, c)| *c)
        };

        // ── ribbons ──────────────────────────────────────────────────────
        // order link ends by the partner's vertical position to limit crossings
        let mut out_off = vec![0.0f64; n];
        let mut in_off = vec![0.0f64; n];
        let mut order: Vec<usize> = (0..links.len()).collect();
        order.sort_by(|a, b| {
            let (la, lb) = (links[*a], links[*b]);
            nodes[la.1]
                .y
                .total_cmp(&nodes[lb.1].y)
                .then(nodes[la.0].y.total_cmp(&nodes[lb.0].y))
        });
        let mut in_order = order.clone();
        in_order.sort_by(|a, b| nodes[links[*a].0].y.total_cmp(&nodes[links[*b].0].y));
        let mut src_y = vec![0.0; links.len()];
        let mut dst_y = vec![0.0; links.len()];
        for &li in &order {
            let (a, _, v) = links[li];
            src_y[li] = nodes[a].y + out_off[a];
            out_off[a] += v * k;
        }
        for &li in &in_order {
            let (_, b, v) = links[li];
            dst_y[li] = nodes[b].y + in_off[b];
            in_off[b] += v * k;
        }
        for (li, &(a, b, v)) in links.iter().enumerate() {
            let (xa, xb) = (col_x(nodes[a].layer) + nw, col_x(nodes[b].layer));
            let h = (v * k).max(1.0);
            let c = darken(color_of(a), 0.55);
            for px in xa..xb {
                let t = (px - xa) as f64 / (xb - xa).max(1) as f64;
                let s = t * t * (3.0 - 2.0 * t);
                let top = src_y[li] + (dst_y[li] - src_y[li]) * s;
                for py in top.round() as i32..(top + h).round().max(top.round() + 1.0) as i32 {
                    cv.set(px, py, c);
                }
            }
        }
        // ── node bars ────────────────────────────────────────────────────
        for (i, nd) in nodes.iter().enumerate() {
            let x = col_x(nd.layer);
            let c = color_of(i);
            for py in nd.y.round() as i32..(nd.y + nd.h).round().max(nd.y.round() + 1.0) as i32 {
                for px in x..x + nw {
                    cv.set(px, py, c);
                }
            }
        }
        cv.render(buf, plot);

        // ── labels ───────────────────────────────────────────────────────
        for nd in &nodes {
            let row = plot.y as i32 + ((nd.y + nd.h / 2.0) / ryd as f64) as i32;
            let row = row.clamp(plot.y as i32, plot.bottom() as i32 - 1) as u16;
            let label = label_of(nd);
            let st = text.add_modifier(Modifier::BOLD);
            let w = label.chars().count() as u16;
            if nd.layer == 0 {
                let l: String = label
                    .chars()
                    .take(left_w.saturating_sub(1) as usize)
                    .collect();
                put_str(buf, inner.x, row, plot.x, &l, st);
            } else if nd.layer == layers - 1 {
                put_str(buf, plot.right() + 1, row, inner.right(), &label, st);
            } else {
                let x = plot.x + (col_x(nd.layer) / rxd as i32) as u16 + self.node_width + 1;
                if x + w <= plot.right() {
                    put_str(buf, x, row, plot.right(), &label, st.fg(Color::White));
                }
            }
        }
        let _ = dim;
    }
}

widget_impls!(Sankey);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for Sankey {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(c: &Sankey, w: u16, h: u16) -> Buffer {
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        c.render(a, &mut b);
        b
    }

    fn links() -> Vec<SankeyLink> {
        vec![
            SankeyLink::new("Salary", "Budget", 70.0),
            SankeyLink::new("Other", "Budget", 30.0),
            SankeyLink::new("Budget", "Rent", 40.0),
            SankeyLink::new("Budget", "Food", 25.0),
            SankeyLink::new("Budget", "Savings", 35.0),
        ]
    }

    #[test]
    fn labels_all_columns_with_totals() {
        let b = render(&Sankey::new(links()), 80, 20);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains("Salary") && t.contains("Rent") && t.contains("Savings"));
        assert!(t.contains("70") && t.contains("100"));
    }

    #[test]
    fn node_colors_apply() {
        let c = Sankey::new(links()).node_color("Salary", Color::Red);
        assert!(render(&c, 80, 20)
            .content()
            .iter()
            .any(|c| c.fg == Color::Red));
    }

    #[test]
    fn values_toggle() {
        let b = render(&Sankey::new(links()).show_values(false), 80, 20);
        let t: String = b.content().iter().map(|c| c.symbol()).collect();
        assert!(t.contains("Salary") && !t.contains("Salary 70"));
    }

    #[test]
    fn thicker_flow_takes_more_cells() {
        let cells = |v: f64| {
            let c = Sankey::new([
                SankeyLink::new("a", "b", v),
                SankeyLink::new("a", "c", 100.0 - v),
            ])
            .node_color("a", Color::Red);
            render(&c, 60, 20)
                .content()
                .iter()
                .filter(|c| c.fg == Color::Red && c.symbol() != " ")
                .count()
        };
        assert!(cells(50.0) > 0);
    }

    #[test]
    fn cycles_and_garbage_do_not_panic_or_hang() {
        let c = Sankey::new([
            SankeyLink::new("a", "b", 1.0),
            SankeyLink::new("b", "c", 1.0),
            SankeyLink::new("c", "a", 1.0),
            SankeyLink::new("a", "a", 5.0),
            SankeyLink::new("x", "y", f64::NAN),
            SankeyLink::new("x", "y", -3.0),
            SankeyLink::new("p", "q", 1e300),
        ]);
        for (w, h) in [(0, 0), (1, 1), (12, 3), (8, 40), (200, 60)] {
            render(&c, w, h);
            render(&Sankey::new(links()), w, h);
        }
        render(&Sankey::new([]), 40, 10);
    }
}
