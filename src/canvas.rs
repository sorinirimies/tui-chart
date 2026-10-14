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
}

impl Marker {
    /// Dots (x, y) per terminal cell.
    pub fn resolution(self) -> (u16, u16) {
        match self {
            Marker::Braille => (2, 4),
            Marker::HalfBlock => (1, 2),
            Marker::Block | Marker::Dot => (1, 1),
        }
    }

    fn glyph(self, mask: u8) -> char {
        match self {
            Marker::Braille => char::from_u32(0x2800 + mask as u32).unwrap_or(' '),
            Marker::HalfBlock => [' ', '▀', '▄', '█'][(mask & 3) as usize],
            Marker::Block => '█',
            Marker::Dot => '•',
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
        }
    }
}

/// A pixel canvas where each terminal cell holds several dots.
#[derive(Debug, Clone)]
pub struct Canvas {
    w: u16,
    h: u16,
    marker: Marker,
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
            mask: vec![0; n],
            color: vec![Color::Reset; n],
        }
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
        self.mask[i] |= self.marker.bit(dx, dy);
        self.color[i] = color;
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
        if self.marker == Marker::Braille {
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
