//! Mouse interactivity for every chart: hit-testing, hover tooltips, click
//! selection and legend toggling.
//!
//! Charts are stateless widgets, so interactivity is a small loop in your app:
//!
//! 1. Render the chart through [`HitMap::capture`]. While it renders, the
//!    chart reports a [`Hit`] for every element it draws (point, bar, slice,
//!    legend entry ...).
//! 2. Feed mouse events into an [`Interaction`] (`on_move`, `on_click`).
//! 3. Pass `interaction.hidden()` to charts that support `.hidden(..)` and call
//!    [`Interaction::render_tooltip`] after the chart.
//!
//! The module is backend-agnostic: it only takes cell coordinates, so it works
//! with crossterm, termion or termwiz mouse events.
//!
//! ```
//! use ratatui::{buffer::Buffer, layout::Rect};
//! use tui_chart::{HitMap, Interaction, BarChart, BarGroup};
//!
//! let area = Rect::new(0, 0, 40, 12);
//! let mut buf = Buffer::empty(area);
//! let chart = BarChart::new(vec![BarGroup::new("a", [3.0]), BarGroup::new("b", [5.0])]);
//! let hits = HitMap::capture(&chart, area, &mut buf);
//!
//! let mut ui = Interaction::default();
//! ui.on_move(10, 8);
//! ui.render_tooltip(&hits, &mut buf, area);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use std::cell::RefCell;
use std::collections::BTreeSet;
use unicode_width::UnicodeWidthStr;

/// What a [`Hit`] describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitKind {
    /// A data element: point, bar, slice, cell, candle, ...
    Point,
    /// A legend entry; clicking toggles its series.
    Legend,
}

/// A rectangle of cells that a chart element occupies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// Bounding box of the element (absolute buffer coordinates).
    pub rect: Rect,
    /// Exact cells for non-rectangular elements (wedges, arcs); empty means
    /// the whole `rect`.
    pub shape: Vec<Rect>,
    /// Element or legend entry.
    pub kind: HitKind,
    /// Series (or group, slice, row) index the element belongs to.
    pub series: usize,
    /// Index of the element inside its series.
    pub index: usize,
    /// Tooltip heading, usually the series or category name.
    pub title: String,
    /// Tooltip body lines.
    pub lines: Vec<String>,
}

thread_local! {
    static RECORDER: RefCell<Option<Vec<Hit>>> = const { RefCell::new(None) };
}

/// Whether a capture is running; charts use this to skip building hit text.
pub(crate) fn recording() -> bool {
    RECORDER.with(|r| r.borrow().is_some())
}

/// Report a data element. `text` is only called while capturing.
pub(crate) fn record_point(
    rect: Rect,
    series: usize,
    index: usize,
    text: impl FnOnce() -> (String, Vec<String>),
) {
    RECORDER.with(|r| {
        if let Some(v) = r.borrow_mut().as_mut() {
            if rect.is_empty() {
                return;
            }
            let (title, lines) = text();
            v.push(Hit {
                rect,
                shape: Vec::new(),
                kind: HitKind::Point,
                series,
                index,
                title,
                lines,
            });
        }
    });
}

/// Report a non-rectangular element made of several rects.
pub(crate) fn record_shape(
    shape: Vec<Rect>,
    series: usize,
    index: usize,
    text: impl FnOnce() -> (String, Vec<String>),
) {
    RECORDER.with(|r| {
        if let Some(v) = r.borrow_mut().as_mut() {
            let Some(first) = shape.first().copied() else {
                return;
            };
            let rect = shape.iter().fold(first, |a, b| a.union(*b));
            let (title, lines) = text();
            v.push(Hit {
                rect,
                shape,
                kind: HitKind::Point,
                series,
                index,
                title,
                lines,
            });
        }
    });
}

/// Report the elements painted into `grid`, a `plot.width × plot.height`
/// row-major map from cell to owning element index. Each owner becomes one
/// [`record_shape`] hit made of horizontal runs.
pub(crate) fn record_grid(
    plot: Rect,
    grid: &[Option<usize>],
    series_of: impl Fn(usize) -> usize,
    text: impl Fn(usize) -> (String, Vec<String>),
) {
    if !recording() || plot.is_empty() {
        return;
    }
    let w = usize::from(plot.width);
    let mut shapes: std::collections::BTreeMap<usize, Vec<Rect>> =
        std::collections::BTreeMap::new();
    for (row, cells) in grid.chunks(w).enumerate() {
        let mut x = 0;
        while x < cells.len() {
            let Some(k) = cells[x] else {
                x += 1;
                continue;
            };
            let start = x;
            while x < cells.len() && cells[x] == Some(k) {
                x += 1;
            }
            shapes.entry(k).or_default().push(Rect::new(
                plot.x + start as u16,
                plot.y + row as u16,
                (x - start) as u16,
                1,
            ));
        }
    }
    for (k, shape) in shapes {
        record_shape(shape, series_of(k), k, || text(k));
    }
}

/// Report a legend entry for `series`.
pub(crate) fn record_legend(rect: Rect, series: usize, name: &str) {
    RECORDER.with(|r| {
        if let Some(v) = r.borrow_mut().as_mut() {
            if rect.is_empty() {
                return;
            }
            v.push(Hit {
                rect,
                shape: Vec::new(),
                kind: HitKind::Legend,
                series,
                index: 0,
                title: name.to_string(),
                lines: Vec::new(),
            });
        }
    });
}

/// The hit regions of one rendered frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HitMap {
    hits: Vec<Hit>,
}

impl HitMap {
    /// Render `widget` into `buf` and collect everything it reports.
    pub fn capture(widget: impl Widget, area: Rect, buf: &mut Buffer) -> Self {
        let prev = RECORDER.with(|r| r.borrow_mut().replace(Vec::new()));
        widget.render(area, buf);
        let hits = RECORDER.with(|r| std::mem::replace(&mut *r.borrow_mut(), prev));
        Self {
            hits: hits.unwrap_or_default(),
        }
    }

    /// All hits in draw order.
    pub fn hits(&self) -> &[Hit] {
        &self.hits
    }

    /// Whether nothing was reported.
    pub fn is_empty(&self) -> bool {
        self.hits.is_empty()
    }

    /// The smallest element whose rect contains `(x, y)`.
    pub fn at(&self, x: u16, y: u16) -> Option<&Hit> {
        self.hits
            .iter()
            .filter(|h| h.kind == HitKind::Point && h.contains(x, y))
            .min_by_key(|h| h.area())
    }

    /// The legend entry under `(x, y)`.
    pub fn legend_at(&self, x: u16, y: u16) -> Option<&Hit> {
        self.hits
            .iter()
            .find(|h| h.kind == HitKind::Legend && contains(h.rect, x, y))
    }

    /// The element nearest to `(x, y)` within `reach` cells (vertical distance
    /// counts double because cells are about twice as tall as wide).
    pub fn nearest(&self, x: u16, y: u16, reach: u16) -> Option<&Hit> {
        let limit = u32::from(reach) * u32::from(reach);
        self.hits
            .iter()
            .filter(|h| h.kind == HitKind::Point)
            .map(|h| (h.dist2(x, y), h))
            .filter(|(d, _)| *d <= limit)
            .min_by_key(|(d, _)| *d)
            .map(|(_, h)| h)
    }

    /// The element to treat as under the pointer: an exact hit, else the
    /// nearest one within 2 cells.
    pub fn pick(&self, x: u16, y: u16) -> Option<&Hit> {
        self.at(x, y).or_else(|| self.nearest(x, y, 2))
    }
}

impl Hit {
    fn rects(&self) -> &[Rect] {
        if self.shape.is_empty() {
            std::slice::from_ref(&self.rect)
        } else {
            &self.shape
        }
    }

    fn contains(&self, x: u16, y: u16) -> bool {
        self.rects().iter().any(|r| contains(*r, x, y))
    }

    fn area(&self) -> u32 {
        self.rects()
            .iter()
            .map(|r| u32::from(r.width) * u32::from(r.height))
            .sum()
    }

    fn dist2(&self, x: u16, y: u16) -> u32 {
        self.rects()
            .iter()
            .map(|r| dist2(*r, x, y))
            .min()
            .unwrap_or(u32::MAX)
    }
}

fn contains(r: Rect, x: u16, y: u16) -> bool {
    x >= r.x && x < r.right() && y >= r.y && y < r.bottom()
}

fn dist2(r: Rect, x: u16, y: u16) -> u32 {
    let dx = if x < r.x {
        r.x - x
    } else if x >= r.right() {
        x - (r.right() - 1)
    } else {
        0
    };
    let dy = if y < r.y {
        r.y - y
    } else if y >= r.bottom() {
        y - (r.bottom() - 1)
    } else {
        0
    };
    u32::from(dx) * u32::from(dx) + 4 * u32::from(dy) * u32::from(dy)
}

/// What a click did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Click {
    /// A legend entry was toggled.
    Toggled {
        /// Series index.
        series: usize,
        /// `true` when the series is now hidden.
        hidden: bool,
    },
    /// A data element was selected (pinned tooltip).
    Selected {
        /// Series index.
        series: usize,
        /// Element index.
        index: usize,
    },
    /// The selection was cleared.
    Cleared,
    /// Nothing under the pointer.
    Nothing,
}

/// Pointer state: hover position, selection and hidden series.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Interaction {
    pos: Option<(u16, u16)>,
    hidden: BTreeSet<usize>,
    selected: Option<(usize, usize)>,
    style: Option<Style>,
}

impl Interaction {
    /// Pointer moved to cell `(x, y)`.
    pub fn on_move(&mut self, x: u16, y: u16) {
        self.pos = Some((x, y));
    }

    /// Pointer left the chart.
    pub fn on_leave(&mut self) {
        self.pos = None;
    }

    /// Pointer position, if inside.
    pub fn position(&self) -> Option<(u16, u16)> {
        self.pos
    }

    /// Primary click at `(x, y)`: toggles a legend entry, or selects / clears.
    pub fn on_click(&mut self, x: u16, y: u16, hits: &HitMap) -> Click {
        self.pos = Some((x, y));
        if let Some(h) = hits.legend_at(x, y) {
            let hidden = self.toggle(h.series);
            return Click::Toggled {
                series: h.series,
                hidden,
            };
        }
        match hits.pick(x, y) {
            Some(h) if self.selected == Some((h.series, h.index)) => {
                self.selected = None;
                Click::Cleared
            }
            Some(h) => {
                self.selected = Some((h.series, h.index));
                Click::Selected {
                    series: h.series,
                    index: h.index,
                }
            }
            None if self.selected.take().is_some() => Click::Cleared,
            None => Click::Nothing,
        }
    }

    /// Toggle visibility of `series`; returns `true` when it is now hidden.
    pub fn toggle(&mut self, series: usize) -> bool {
        if !self.hidden.remove(&series) {
            self.hidden.insert(series);
            return true;
        }
        false
    }

    /// Hidden series indices, for charts' `.hidden(..)`.
    pub fn hidden(&self) -> Vec<usize> {
        self.hidden.iter().copied().collect()
    }

    /// Whether `series` is hidden.
    pub fn is_hidden(&self, series: usize) -> bool {
        self.hidden.contains(&series)
    }

    /// Show every series again and drop the selection.
    pub fn reset(&mut self) {
        self.hidden.clear();
        self.selected = None;
    }

    /// The pinned (clicked) element, `(series, index)`.
    pub fn selected(&self) -> Option<(usize, usize)> {
        self.selected
    }

    /// Tooltip style (default: white on dark gray).
    #[must_use]
    pub fn tooltip_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// The element currently under the pointer.
    pub fn hovered<'a>(&self, hits: &'a HitMap) -> Option<&'a Hit> {
        let (x, y) = self.pos?;
        hits.pick(x, y)
    }

    /// Highlight the hovered / selected element and draw its tooltip, clamped
    /// to `bounds`. Call after rendering the chart.
    pub fn render_tooltip(&self, hits: &HitMap, buf: &mut Buffer, bounds: Rect) {
        let bounds = bounds.intersection(buf.area);
        let selected = self.selected.and_then(|(s, i)| {
            hits.hits
                .iter()
                .find(|h| h.kind == HitKind::Point && h.series == s && h.index == i)
        });
        for h in selected.into_iter().chain(self.hovered(hits)) {
            for r in h.rects() {
                highlight(buf, r.intersection(bounds));
            }
        }
        let Some(h) = self.hovered(hits).or(selected) else {
            return;
        };
        let (px, py) = self.pos.unwrap_or((h.rect.x, h.rect.y));
        let mut lines: Vec<&str> = Vec::new();
        if !h.title.is_empty() {
            lines.push(&h.title);
        }
        lines.extend(h.lines.iter().map(String::as_str));
        if lines.is_empty() || bounds.is_empty() {
            return;
        }
        let w = lines
            .iter()
            .map(|l| UnicodeWidthStr::width(*l))
            .max()
            .unwrap_or(0) as u16
            + 2;
        let hgt = lines.len() as u16;
        let (w, hgt) = (w.min(bounds.width), hgt.min(bounds.height));
        let mut x = px.saturating_add(2);
        if x + w > bounds.right() {
            x = px.saturating_sub(w + 1).max(bounds.x);
        }
        let mut y = py.saturating_add(1);
        if y + hgt > bounds.bottom() {
            y = py.saturating_sub(hgt).max(bounds.y);
        }
        let x = x.clamp(bounds.x, bounds.right().saturating_sub(w));
        let y = y.clamp(bounds.y, bounds.bottom().saturating_sub(hgt));
        let style = self
            .style
            .unwrap_or_else(|| Style::default().fg(Color::White).bg(Color::DarkGray));
        for (i, l) in lines.iter().take(usize::from(hgt)).enumerate() {
            let row = y + i as u16;
            for cx in x..x + w {
                buf[(cx, row)].set_char(' ').set_style(style);
            }
            let st = if i == 0 && !h.title.is_empty() {
                style.add_modifier(Modifier::BOLD)
            } else {
                style
            };
            buf.set_stringn(x + 1, row, l, usize::from(w.saturating_sub(2)), st);
        }
    }
}

fn highlight(buf: &mut Buffer, r: Rect) {
    for y in r.top()..r.bottom() {
        for x in r.left()..r.right() {
            let c = &mut buf[(x, y)];
            c.modifier.insert(Modifier::REVERSED);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hm() -> HitMap {
        let area = Rect::new(0, 0, 20, 8);
        let mut buf = Buffer::empty(area);
        struct W;
        impl Widget for W {
            fn render(self, _: Rect, _: &mut Buffer) {
                record_point(Rect::new(2, 2, 3, 3), 0, 0, || {
                    ("a".into(), vec!["v 1".into()])
                });
                record_point(Rect::new(10, 2, 1, 1), 1, 4, || ("b".into(), vec![]));
                record_legend(Rect::new(0, 7, 6, 1), 1, "b");
            }
        }
        HitMap::capture(W, area, &mut buf)
    }

    #[test]
    fn capture_collects_and_is_scoped() {
        let h = hm();
        assert_eq!(h.hits().len(), 3);
        assert!(!recording());
        assert!(!h.is_empty());
    }

    #[test]
    fn at_nearest_pick_legend() {
        let h = hm();
        assert_eq!(h.at(3, 3).unwrap().series, 0);
        assert!(h.at(8, 3).is_none());
        assert_eq!(h.nearest(9, 2, 2).unwrap().series, 1);
        assert!(h.nearest(15, 2, 2).is_none());
        assert_eq!(h.pick(8, 2).unwrap().index, 4);
        assert_eq!(h.legend_at(1, 7).unwrap().series, 1);
        assert!(h.legend_at(1, 5).is_none());
    }

    #[test]
    fn shapes_hit_only_their_cells() {
        let area = Rect::new(0, 0, 10, 4);
        let grid: Vec<Option<usize>> = (0..40)
            .map(|i| match (i / 10, i % 10) {
                (0, 0..=2) => Some(0),
                (1, 1..=2) => Some(0),
                (_, 8..=9) => Some(1),
                _ => None,
            })
            .collect();
        struct W(Vec<Option<usize>>);
        impl Widget for W {
            fn render(self, a: Rect, _: &mut Buffer) {
                record_grid(a, &self.0, |k| k, |k| (format!("k{k}"), vec![]));
            }
        }
        let mut buf = Buffer::empty(area);
        let h = HitMap::capture(W(grid), area, &mut buf);
        assert_eq!(h.hits().len(), 2);
        assert_eq!(h.at(1, 0).unwrap().title, "k0");
        assert!(h.at(5, 3).is_none(), "bbox corner outside the wedge");
        assert_eq!(h.at(9, 3).unwrap().title, "k1");
        assert_eq!(h.at(2, 1).unwrap().index, 0);
        assert!(h.at(0, 1).is_none());
    }

    #[test]
    fn click_selects_clears_and_toggles_legend() {
        let h = hm();
        let mut ui = Interaction::default();
        assert_eq!(
            ui.on_click(3, 3, &h),
            Click::Selected {
                series: 0,
                index: 0
            }
        );
        assert_eq!(ui.selected(), Some((0, 0)));
        assert_eq!(ui.on_click(3, 3, &h), Click::Cleared);
        assert_eq!(ui.on_click(18, 0, &h), Click::Nothing);
        assert_eq!(
            ui.on_click(1, 7, &h),
            Click::Toggled {
                series: 1,
                hidden: true
            }
        );
        assert_eq!(ui.hidden(), vec![1]);
        assert!(ui.is_hidden(1));
        assert_eq!(
            ui.on_click(1, 7, &h),
            Click::Toggled {
                series: 1,
                hidden: false
            }
        );
        ui.on_click(3, 3, &h);
        assert_eq!(ui.on_click(18, 0, &h), Click::Cleared);
        ui.toggle(2);
        ui.reset();
        assert!(ui.hidden().is_empty() && ui.selected().is_none());
    }

    #[test]
    fn tooltip_draws_clamped_and_highlights() {
        let h = hm();
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 8));
        let mut ui = Interaction::default();
        ui.on_move(3, 3);
        ui.render_tooltip(&h, &mut buf, Rect::new(0, 0, 20, 8));
        assert!(
            buf[(3, 3)].modifier.contains(Modifier::REVERSED) || buf[(3, 4)].bg == Color::DarkGray
        );
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains('a') && text.contains("v 1"));
        // near the corner: still inside the buffer
        ui.on_move(19, 7);
        let h2 = HitMap {
            hits: vec![Hit {
                rect: Rect::new(19, 7, 1, 1),
                shape: vec![],
                kind: HitKind::Point,
                series: 0,
                index: 0,
                title: "wide tooltip title".into(),
                lines: vec!["line".into()],
            }],
        };
        ui.render_tooltip(&h2, &mut buf, Rect::new(0, 0, 20, 8));
        ui.on_leave();
        assert!(ui.position().is_none());
        ui.render_tooltip(&h2, &mut buf, Rect::new(0, 0, 0, 0));
    }
}
