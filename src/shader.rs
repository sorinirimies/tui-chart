//! GPU shader backdrops via [`tui-shader`](https://crates.io/crates/tui-shader)
//! (feature `shader`).
//!
//! A [`ShaderBackdrop`] runs a WGSL fragment shader and applies its colors to
//! the **background** of the cells under a chart. The chart's glyphs stay
//! intact, so any widget can sit on an animated neon grid, aurora or plasma.
//!
//! Needs a GPU adapter (Metal, Vulkan, DX12). Creation returns an error for
//! invalid WGSL; with no adapter the underlying crate may panic, so keep the
//! CPU [`Fx`](crate::Fx) effects as the fallback.
//!
//! ```no_run
//! use tui_chart::{ShaderBackdrop, ShaderPreset, Spectrum};
//! # use ratatui::{buffer::Buffer, layout::Rect};
//! let mut bg = ShaderBackdrop::new(ShaderPreset::Grid).unwrap().intensity(0.7);
//! let area = Rect::new(0, 0, 60, 20);
//! let mut buf = Buffer::empty(area);
//! bg.render(Spectrum::new([0.3, 0.9, 0.6]), area, &mut buf);
//! ```

use crate::util::mix;
use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use std::error::Error;
use tui_shader::{ShaderCanvas, ShaderCanvasState, WgslShader};

/// Built-in backdrop shaders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShaderPreset {
    /// Scrolling synthwave horizon grid.
    #[default]
    Grid,
    /// Flowing aurora ribbons.
    Aurora,
    /// Magenta / cyan plasma.
    Plasma,
}

impl ShaderPreset {
    /// Every preset.
    pub const ALL: [Self; 3] = [Self::Grid, Self::Aurora, Self::Plasma];

    /// The WGSL source of the preset.
    pub fn source(self) -> &'static str {
        match self {
            Self::Grid => include_str!("shaders/grid.wgsl"),
            Self::Aurora => include_str!("shaders/aurora.wgsl"),
            Self::Plasma => include_str!("shaders/plasma.wgsl"),
        }
    }
}

/// An animated GPU background for any widget.
pub struct ShaderBackdrop {
    state: ShaderCanvasState,
    intensity: f64,
}

impl ShaderBackdrop {
    /// A backdrop from a built-in preset.
    pub fn new(preset: ShaderPreset) -> Result<Self, Box<dyn Error>> {
        Self::custom(preset.source())
    }

    /// A backdrop from your own WGSL.
    ///
    /// The fragment shader gets `@location(0) uv: vec2<f32>`, time at
    /// `@group(0) @binding(0)` and the rect at `@binding(1)` (both `vec4<f32>`).
    pub fn custom(wgsl: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            state: ShaderCanvasState::new(WgslShader::Source(wgsl))?,
            intensity: 0.6,
        })
    }

    /// Blend of shader and existing background, `0..=1` (default `0.6`).
    #[must_use]
    pub fn intensity(mut self, intensity: f64) -> Self {
        self.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Paint the shader into the backgrounds of `area` (glyphs are kept).
    pub fn apply(&mut self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        let mut scratch = Buffer::empty(area);
        ratatui::widgets::StatefulWidget::render(
            ShaderCanvas::new(),
            area,
            &mut scratch,
            &mut self.state,
        );
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let sbg = scratch[(x, y)].bg;
                let cell = &mut buf[(x, y)];
                let base = if cell.bg == Color::Reset {
                    Color::Black
                } else {
                    cell.bg
                };
                cell.bg = mix(base, sbg, self.intensity);
            }
        }
    }

    /// Render `widget`, then lay the shader behind it.
    pub fn render<W: Widget>(&mut self, widget: W, area: Rect, buf: &mut Buffer) {
        widget.render(area, buf);
        self.apply(area, buf);
    }
}

impl std::fmt::Debug for ShaderBackdrop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderBackdrop")
            .field("intensity", &self.intensity)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_wgsl_is_an_error_not_a_panic() {
        assert!(ShaderBackdrop::custom("this is not wgsl").is_err());
    }

    #[test]
    fn presets_have_sources() {
        for p in ShaderPreset::ALL {
            assert!(p.source().contains("@fragment"), "{p:?}");
        }
    }

    #[test]
    #[ignore = "needs a GPU adapter"]
    fn backdrop_paints_backgrounds_and_keeps_glyphs() {
        let a = Rect::new(0, 0, 20, 8);
        let mut buf = Buffer::empty(a);
        buf[(3, 3)].set_char('█');
        let mut bg = ShaderBackdrop::new(ShaderPreset::Grid)
            .unwrap()
            .intensity(1.0);
        bg.apply(a, &mut buf);
        assert_eq!(buf[(3, 3)].symbol(), "█");
        assert!(buf
            .content()
            .iter()
            .any(|c| matches!(c.bg, Color::Rgb(r, g, b) if r | g | b != 0)));
    }
}
