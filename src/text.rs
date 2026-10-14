//! Text helpers: clipping and clipped drawing.

use ratatui::{buffer::Buffer, style::Style};

/// First `n` chars of `s`.
pub(crate) fn trunc(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Draw `s` at `(x, y)` char by char, stopping before column `max_x`.
pub(crate) fn put_str(buf: &mut Buffer, x: u16, y: u16, max_x: u16, s: &str, style: Style) {
    for (i, ch) in s.chars().enumerate() {
        let cx = x as u32 + i as u32;
        if cx >= max_x as u32 {
            break;
        }
        buf[(cx as u16, y)].set_char(ch).set_style(style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn trunc_chars() {
        assert_eq!(trunc("héllo", 2), "hé");
        assert_eq!(trunc("ab", 9), "ab");
        assert_eq!(trunc("ab", 0), "");
    }

    #[test]
    fn put_str_clips() {
        let mut b = Buffer::empty(Rect::new(0, 0, 5, 1));
        put_str(&mut b, 2, 0, 4, "abcdef", Style::default());
        assert_eq!(b[(2, 0)].symbol(), "a");
        assert_eq!(b[(3, 0)].symbol(), "b");
        assert_eq!(b[(4, 0)].symbol(), " ");
    }
}
