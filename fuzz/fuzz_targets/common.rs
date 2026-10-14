//! Shared helpers: capped arbitrary inputs mapped onto tui-chart types.
#![allow(dead_code)]

use arbitrary::Arbitrary;
use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use tui_chart::*;

pub const MAX_POINTS: usize = 2000;
pub const MAX_W: u16 = 200;
pub const MAX_H: u16 = 100;

#[derive(Arbitrary, Debug, Clone, Copy)]
pub struct Size {
    pub w: u8,
    pub h: u8,
    pub x: u8,
    pub y: u8,
}

impl Size {
    /// Area clamped to <= 200x100.
    pub fn area(&self) -> Rect {
        Rect::new(
            self.x as u16 % 8,
            self.y as u16 % 8,
            (self.w as u16) % (MAX_W + 1),
            (self.h as u16) % (MAX_H + 1),
        )
    }
}

/// Render any widget into a buffer covering `size` (plus the offset).
pub fn render<W: Widget>(w: W, size: Size) {
    let a = size.area();
    let full = Rect::new(0, 0, a.x + a.width, a.y + a.height);
    let mut buf = Buffer::empty(full);
    w.render(a, &mut buf);
}

#[derive(Arbitrary, Debug, Clone, Copy)]
pub struct Col(pub u8, pub u8, pub u8, pub u8);

impl Col {
    pub fn get(self) -> Color {
        match self.0 % 4 {
            0 => Color::Rgb(self.1, self.2, self.3),
            1 => Color::Indexed(self.1),
            2 => Color::Reset,
            _ => Color::Cyan,
        }
    }
}

#[derive(Arbitrary, Debug, Clone, Copy)]
pub enum MarkerIn {
    Braille,
    HalfBlock,
    Block,
    Dot,
    Sextant,
}

impl MarkerIn {
    pub fn get(self) -> Marker {
        match self {
            Self::Braille => Marker::Braille,
            Self::HalfBlock => Marker::HalfBlock,
            Self::Block => Marker::Block,
            Self::Dot => Marker::Dot,
            Self::Sextant => Marker::Sextant,
        }
    }
}

#[derive(Arbitrary, Debug, Clone)]
pub struct AxisIn {
    pub title: Option<String>,
    pub bounds: Option<(f64, f64)>,
    pub ticks: Option<u8>,
    pub grid: bool,
    pub grid_kind: u8,
    pub labels: Option<Vec<String>>,
}

impl AxisIn {
    pub fn get(&self) -> Axis {
        let mut a = Axis::new().grid(self.grid).grid_lines(match self.grid_kind % 3 {
            0 => GridStyle::Dotted,
            1 => GridStyle::Dashed,
            _ => GridStyle::Solid,
        });
        if let Some(t) = &self.title {
            a = a.title(t.clone());
        }
        if let Some((lo, hi)) = self.bounds {
            a = a.bounds(lo, hi);
        }
        if let Some(t) = self.ticks {
            a = a.ticks(t as usize);
        }
        if let Some(l) = &self.labels {
            a = a.labels(l.iter().take(64).cloned());
        }
        a
    }
}

#[derive(Arbitrary, Debug, Clone, Copy)]
pub struct CandleIn(pub f64, pub f64, pub f64, pub f64, pub f64);

pub fn candles(v: &[CandleIn]) -> Vec<Candle> {
    v.iter()
        .take(MAX_POINTS)
        .map(|c| Candle::new(c.0, c.1, c.2, c.3).volume(c.4))
        .collect()
}

pub fn theme(i: u8) -> Theme {
    let all = Theme::all();
    all[i as usize % all.len()].clone()
}

pub fn cap<T: Clone>(v: &[T]) -> Vec<T> {
    v.iter().take(MAX_POINTS).cloned().collect()
}
