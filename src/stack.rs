//! Layout helper that splits an area into linked chart panes.

use ratatui::layout::Rect;

/// Direction in which panes are stacked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StackDirection {
    /// Panes stacked top to bottom (shared x axis, e.g. price + volume + indicator).
    #[default]
    Vertical,
    /// Panes placed left to right.
    Horizontal,
}

/// Splits an area into panes sized by ratio weights.
///
/// ```
/// use ratatui::layout::Rect;
/// use tui_chart::ChartStack;
///
/// let panes = ChartStack::vertical().weights([3, 1]).gap(1).split(Rect::new(0, 0, 40, 21));
/// assert_eq!(panes.len(), 2);
/// assert_eq!(panes[0].height, 15);
/// assert_eq!(panes[1].height, 5);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ChartStack {
    direction: StackDirection,
    weights: Vec<u16>,
    gap: u16,
}

impl ChartStack {
    /// New stack in the given direction; add panes with [`pane`](Self::pane).
    pub fn new(direction: StackDirection) -> Self {
        Self {
            direction,
            ..Default::default()
        }
    }

    /// Vertical stack (panes top to bottom).
    pub fn vertical() -> Self {
        Self::new(StackDirection::Vertical)
    }

    /// Horizontal stack (panes left to right).
    pub fn horizontal() -> Self {
        Self::new(StackDirection::Horizontal)
    }

    /// Append a pane with a ratio weight (0 is treated as 1).
    pub fn pane(mut self, weight: u16) -> Self {
        self.weights.push(weight.max(1));
        self
    }

    /// Replace all panes with the given weights.
    pub fn weights(mut self, weights: impl IntoIterator<Item = u16>) -> Self {
        self.weights = weights.into_iter().map(|w| w.max(1)).collect();
        self
    }

    setter!(
        /// Empty cells between panes (default 0).
        gap: u16
    );

    /// Number of panes.
    pub fn len(&self) -> usize {
        self.weights.len()
    }

    /// True when no panes are configured.
    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }

    /// Split `area` into one rect per pane. Sizes are proportional to the weights
    /// (largest-remainder rounding, so they always fill the area exactly); when the
    /// area is too small, trailing panes become zero-sized instead of panicking.
    pub fn split(&self, area: Rect) -> Vec<Rect> {
        let n = self.weights.len();
        if n == 0 {
            return vec![];
        }
        let total_len = match self.direction {
            StackDirection::Vertical => area.height,
            StackDirection::Horizontal => area.width,
        } as u32;
        let gaps = (self.gap as u32)
            .saturating_mul(n as u32 - 1)
            .min(total_len);
        let avail = total_len - gaps;
        let wsum: u32 = self.weights.iter().map(|&w| w as u32).sum();
        let mut sizes: Vec<u32> = self
            .weights
            .iter()
            .map(|&w| avail * w as u32 / wsum)
            .collect();
        let rest = avail - sizes.iter().sum::<u32>();
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(avail * self.weights[i] as u32 % wsum));
        for &i in order.iter().cycle().take(rest as usize) {
            sizes[i] += 1;
        }
        let mut pos = 0u32;
        sizes
            .iter()
            .enumerate()
            .map(|(i, &sz)| {
                let g = if i + 1 < n { self.gap as u32 } else { 0 };
                let r = match self.direction {
                    StackDirection::Vertical => Rect::new(
                        area.x,
                        area.y + pos.min(area.height as u32) as u16,
                        area.width,
                        sz as u16,
                    ),
                    StackDirection::Horizontal => Rect::new(
                        area.x + pos.min(area.width as u32) as u16,
                        area.y,
                        sz as u16,
                        area.height,
                    ),
                };
                pos += sz + g;
                r
            })
            .collect()
    }

    /// Like [`split`](Self::split) but each pane is divided into
    /// `(label_gutter, content)` where the gutter is the left `gutter` columns
    /// (clamped to the pane width). Rendering every chart into its `content` rect
    /// with y-axis labels drawn in a matching gutter keeps plot areas x-aligned
    /// across panes. Use [`shared_gutter`] to compute the width.
    pub fn split_aligned(&self, area: Rect, gutter: u16) -> Vec<(Rect, Rect)> {
        self.split(area)
            .into_iter()
            .map(|r| {
                let g = gutter.min(r.width);
                (
                    Rect::new(r.x, r.y, g, r.height),
                    Rect::new(r.x + g, r.y, r.width - g, r.height),
                )
            })
            .collect()
    }
}

/// Shared gutter width for x-aligned panes: the widest of the given label widths.
pub fn shared_gutter(widths: impl IntoIterator<Item = u16>) -> u16 {
    widths.into_iter().max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_area_exactly() {
        for h in 0..60u16 {
            let r = ChartStack::vertical()
                .weights([3, 2, 1])
                .gap(1)
                .split(Rect::new(2, 3, 10, h));
            assert_eq!(r.len(), 3);
            let used: u32 = r.iter().map(|r| r.height as u32).sum();
            let gaps = 2.min(h as u32);
            assert_eq!(used + gaps, h as u32, "h={h}");
            for r in &r {
                assert!(r.bottom() <= 3 + h);
            }
            assert!(r.windows(2).all(|w| w[0].bottom() <= w[1].y));
        }
    }

    #[test]
    fn proportions() {
        let r = ChartStack::vertical()
            .pane(3)
            .pane(1)
            .split(Rect::new(0, 0, 5, 20));
        assert_eq!((r[0].height, r[1].height), (15, 5));
        assert_eq!(r[1].y, 15);
    }

    #[test]
    fn horizontal_and_gap() {
        let r = ChartStack::horizontal()
            .weights([1, 1])
            .gap(2)
            .split(Rect::new(0, 0, 22, 4));
        assert_eq!(r[0], Rect::new(0, 0, 10, 4));
        assert_eq!(r[1], Rect::new(12, 0, 10, 4));
    }

    #[test]
    fn empty_and_tiny() {
        assert!(ChartStack::vertical()
            .split(Rect::new(0, 0, 5, 5))
            .is_empty());
        assert!(ChartStack::vertical().is_empty());
        let r = ChartStack::vertical()
            .weights([1, 1, 1])
            .gap(5)
            .split(Rect::new(0, 0, 0, 0));
        assert!(r.iter().all(|r| r.area() == 0));
        let r = ChartStack::vertical()
            .weights([0, u16::MAX])
            .split(Rect::new(0, 0, 3, 7));
        assert_eq!(r.iter().map(|r| r.height).sum::<u16>(), 7);
    }

    #[test]
    fn aligned_gutter() {
        let s = ChartStack::vertical().weights([2, 1]);
        let g = shared_gutter([4, 7, 5]);
        assert_eq!(g, 7);
        let p = s.split_aligned(Rect::new(0, 0, 30, 12), g);
        assert!(p
            .iter()
            .all(|(l, c)| l.width == 7 && c.x == 7 && c.width == 23));
        let p = s.split_aligned(Rect::new(0, 0, 3, 12), g);
        assert_eq!(p[0].1.width, 0);
        assert_eq!(shared_gutter([]), 0);
    }
}
