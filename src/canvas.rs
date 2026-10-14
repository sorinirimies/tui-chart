//! Sub-cell resolution drawing surface used by the line/area/scatter chart.

use ratatui::{buffer::Buffer, layout::Rect, style::Color};

/// Glyph family used to draw plotted points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Marker {
    /// Braille patterns: 2×4 dots per cell (highest resolution).
    #[default]
    Braille,
    /// Half blocks `▀▄█`: 1×2 per cell.
    HalfBlock,
    /// Full block `█`: 1×1 per cell.
    Block,
    /// Bullet `•`: 1×1 per cell.
    Dot,
    /// Sextants (Unicode "Symbols for Legacy Computing", U+1FB00..U+1FB3B):
    /// 2×3 dots per cell.
    ///
    /// Font caveat: many terminal fonts lack these glyphs (they render as
    /// tofu or blanks), so this marker is opt-in. Braille is the default.
    Sextant,
}

/// Which colour wins when several dots with different colours share a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OverlapPolicy {
    /// The most recently set dot decides the cell colour (default).
    #[default]
    LastWins,
    /// The first dot set in a cell keeps its colour.
    FirstWins,
}

impl Marker {
    /// All markers in cycle order.
    pub const ALL: [Marker; 5] = [
        Marker::Braille,
        Marker::HalfBlock,
        Marker::Block,
        Marker::Dot,
        Marker::Sextant,
    ];

    /// Next marker in [`Marker::ALL`] (wraps); handy for demo key bindings.
    pub fn next(self) -> Marker {
        let i = Self::ALL.iter().position(|m| *m == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    /// Dots (x, y) per terminal cell.
    pub fn resolution(self) -> (u16, u16) {
        match self {
            Marker::Braille => (2, 4),
            Marker::HalfBlock => (1, 2),
            Marker::Block | Marker::Dot => (1, 1),
            Marker::Sextant => (2, 3),
        }
    }

    fn glyph(self, mask: u8) -> char {
        match self {
            Marker::Braille => char::from_u32(0x2800 + mask as u32).unwrap_or(' '),
            Marker::HalfBlock => [' ', '▀', '▄', '█'][(mask & 3) as usize],
            Marker::Block => '█',
            Marker::Dot => '•',
            Marker::Sextant => match mask & 63 {
                0 => ' ',
                21 => '▌',
                42 => '▐',
                63 => '█',
                m => {
                    let m = m as u32;
                    let skipped = (m > 21) as u32 + (m > 42) as u32;
                    char::from_u32(0x1FB00 + m - 1 - skipped).unwrap_or(' ')
                }
            },
        }
    }

    fn bit(self, dx: u16, dy: u16) -> u8 {
        match self {
            Marker::Braille => {
                const BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
                BITS[dx as usize][dy as usize]
            }
            Marker::HalfBlock => 1 << dy,
            Marker::Block | Marker::Dot => 1,
            Marker::Sextant => 1 << (dy * 2 + dx),
        }
    }
}

/// A pixel canvas where each terminal cell holds several dots.
#[derive(Debug, Clone)]
pub struct Canvas {
    w: u16,
    h: u16,
    marker: Marker,
    overlap: OverlapPolicy,
    mask: Vec<u8>,
    color: Vec<Color>,
}

impl Canvas {
    /// Create a canvas covering `w × h` terminal cells.
    pub fn new(w: u16, h: u16, marker: Marker) -> Self {
        let n = w as usize * h as usize;
        Self {
            w,
            h,
            marker,
            overlap: OverlapPolicy::default(),
            mask: vec![0; n],
            color: vec![Color::Reset; n],
        }
    }

    /// Set the per-cell colour policy for overlapping dots.
    pub fn set_overlap(&mut self, policy: OverlapPolicy) {
        self.overlap = policy;
    }

    /// Builder form of [`Canvas::set_overlap`].
    pub fn with_overlap(mut self, policy: OverlapPolicy) -> Self {
        self.overlap = policy;
        self
    }

    /// Marker of this canvas.
    pub fn marker(&self) -> Marker {
        self.marker
    }

    /// Width in dots.
    pub fn px_w(&self) -> i32 {
        self.w as i32 * self.marker.resolution().0 as i32
    }

    /// Height in dots.
    pub fn px_h(&self) -> i32 {
        self.h as i32 * self.marker.resolution().1 as i32
    }

    /// Set one dot. Out-of-range coordinates are ignored.
    pub fn set(&mut self, px: i32, py: i32, color: Color) {
        if px < 0 || py < 0 || px >= self.px_w() || py >= self.px_h() {
            return;
        }
        let (rx, ry) = self.marker.resolution();
        let (cx, cy) = (px as u16 / rx, py as u16 / ry);
        let (dx, dy) = (px as u16 % rx, py as u16 % ry);
        let i = cy as usize * self.w as usize + cx as usize;
        let was_empty = self.mask[i] == 0;
        self.mask[i] |= self.marker.bit(dx, dy);
        if was_empty || self.overlap == OverlapPolicy::LastWins {
            self.color[i] = color;
        }
    }

    /// Whether a dot is set.
    pub fn is_set(&self, px: i32, py: i32) -> bool {
        if px < 0 || py < 0 || px >= self.px_w() || py >= self.px_h() {
            return false;
        }
        let (rx, ry) = self.marker.resolution();
        let i = (py as u16 / ry) as usize * self.w as usize + (px as u16 / rx) as usize;
        self.mask[i] & self.marker.bit(px as u16 % rx, py as u16 % ry) != 0
    }

    /// A slightly bigger point (2×2 dots on braille) for scatter plots.
    pub fn blob(&mut self, px: i32, py: i32, color: Color) {
        self.set(px, py, color);
        if matches!(self.marker, Marker::Braille | Marker::Sextant) {
            self.set(px + 1, py, color);
            self.set(px, py + 1, color);
            self.set(px + 1, py + 1, color);
        }
    }

    /// Bresenham line between two dots.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let (mut x0, mut y0) = (x0 as i64, y0 as i64);
        let (x1, y1) = (x1 as i64, y1 as i64);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set(x0 as i32, y0 as i32, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Paint non-empty cells into `buf` at `origin` (top-left of the plot area).
    pub fn render(&self, buf: &mut Buffer, origin: Rect) {
        for cy in 0..self.h.min(origin.height) {
            for cx in 0..self.w.min(origin.width) {
                let i = cy as usize * self.w as usize + cx as usize;
                if self.mask[i] == 0 {
                    continue;
                }
                if let Some(cell) = buf.cell_mut((origin.x + cx, origin.y + cy)) {
                    cell.set_char(self.marker.glyph(self.mask[i]));
                    cell.set_fg(self.color[i]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_per_marker() {
        assert_eq!(Marker::Braille.resolution(), (2, 4));
        assert_eq!(Marker::HalfBlock.resolution(), (1, 2));
        assert_eq!(Marker::Block.resolution(), (1, 1));
        assert_eq!(Marker::Dot.resolution(), (1, 1));
        assert_eq!(Marker::Sextant.resolution(), (2, 3));
    }

    #[test]
    fn marker_cycle_wraps() {
        let mut m = Marker::Braille;
        for _ in 0..Marker::ALL.len() {
            m = m.next();
        }
        assert_eq!(m, Marker::Braille);
    }

    #[test]
    fn sextant_glyphs() {
        let g = |m: u8| Marker::Sextant.glyph(m);
        assert_eq!(g(0), ' ');
        assert_eq!(g(1), '\u{1FB00}');
        assert_eq!(g(20), '\u{1FB13}');
        assert_eq!(g(21), '▌');
        assert_eq!(g(22), '\u{1FB14}');
        assert_eq!(g(42), '▐');
        assert_eq!(g(43), '\u{1FB28}');
        assert_eq!(g(62), '\u{1FB3B}');
        assert_eq!(g(63), '█');
        let mut c = Canvas::new(1, 1, Marker::Sextant);
        assert_eq!((c.px_w(), c.px_h()), (2, 3));
        for y in 0..3 {
            c.set(0, y, Color::Red);
        }
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        c.render(&mut buf, Rect::new(0, 0, 1, 1));
        assert_eq!(buf[(0, 0)].symbol(), "▌");
    }

    #[test]
    fn overlap_policy() {
        let mut c = Canvas::new(1, 1, Marker::Braille);
        c.set(0, 0, Color::Red);
        c.set(1, 0, Color::Blue);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        c.render(&mut buf, Rect::new(0, 0, 1, 1));
        assert_eq!(buf[(0, 0)].fg, Color::Blue);
        let mut c = Canvas::new(1, 1, Marker::Braille);
        c.set_overlap(OverlapPolicy::FirstWins);
        c.set(0, 0, Color::Red);
        c.set(1, 0, Color::Blue);
        c.render(&mut buf, Rect::new(0, 0, 1, 1));
        assert_eq!(buf[(0, 0)].fg, Color::Red);
    }

    #[test]
    fn set_and_query() {
        let mut c = Canvas::new(4, 2, Marker::Braille);
        assert_eq!((c.px_w(), c.px_h()), (8, 8));
        c.set(3, 5, Color::Red);
        assert!(c.is_set(3, 5));
        assert!(!c.is_set(2, 5));
        c.set(-1, 0, Color::Red);
        c.set(8, 0, Color::Red);
        assert!(!c.is_set(8, 0));
    }

    #[test]
    fn braille_glyph() {
        let mut c = Canvas::new(1, 1, Marker::Braille);
        c.set(0, 0, Color::Red);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        c.render(&mut buf, Rect::new(0, 0, 1, 1));
        assert_eq!(buf[(0, 0)].symbol(), "\u{2801}");
        c.set(1, 3, Color::Red);
        c.render(&mut buf, Rect::new(0, 0, 1, 1));
        assert_eq!(buf[(0, 0)].symbol(), "\u{2881}");
    }

    #[test]
    fn half_block_glyph() {
        let mut c = Canvas::new(1, 1, Marker::HalfBlock);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        c.set(0, 0, Color::Red);
        c.render(&mut buf, Rect::new(0, 0, 1, 1));
        assert_eq!(buf[(0, 0)].symbol(), "▀");
        c.set(0, 1, Color::Red);
        c.render(&mut buf, Rect::new(0, 0, 1, 1));
        assert_eq!(buf[(0, 0)].symbol(), "█");
    }

    #[test]
    fn line_connects_endpoints() {
        let mut c = Canvas::new(10, 5, Marker::Braille);
        c.line(0, 0, 19, 19, Color::Green);
        assert!(c.is_set(0, 0));
        assert!(c.is_set(19, 19));
        assert!(c.is_set(10, 10));
    }

    #[test]
    fn line_with_huge_coords_terminates() {
        let mut c = Canvas::new(4, 4, Marker::Block);
        c.line(-10_000, -10_000, 10_000, 10_000, Color::Green);
        assert!(c.is_set(2, 2));
    }

    #[test]
    fn blob_is_bigger_on_braille() {
        let mut c = Canvas::new(2, 2, Marker::Braille);
        c.blob(1, 1, Color::Red);
        assert!(c.is_set(2, 2));
        let mut d = Canvas::new(2, 2, Marker::Block);
        d.blob(1, 1, Color::Red);
        assert!(!d.is_set(0, 0));
    }
}
