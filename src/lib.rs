//! # tui-chart
//!
//! Chart widgets for [ratatui](https://ratatui.rs).
//!
//! ## Widgets
//!
//! - [`LineChart`] – line / step / area / scatter, multi-series, secondary y axis
//! - [`CandlestickChart`] – candles, Heikin-Ashi, OHLC, volume, indicator panes
//! - [`BarChart`] – vertical/horizontal, grouped/stacked
//! - [`Histogram`], [`Heatmap`], [`Sparkline`], [`Gauge`], [`RadarChart`]
//! - [`BoxPlot`], [`Waterfall`], [`Gantt`], [`DepthChart`], [`RenkoChart`], [`Treemap`]
//! - [`ChartStack`] – stack charts with a shared gutter
//!
//! ## Features
//!
//! - [`Annotation`]s (lines, regions, labels) and an interactive [`Crosshair`]
//! - log scale (`y_log`), spline smoothing, error bars and bands
//! - indicators: [`sma`], [`ema`], [`bollinger`], [`rsi`], [`macd`], [`atr`],
//!   [`stochastic`], [`vwap`]
//! - streaming with [`RingBuffer`] / [`Stream`]
//! - time axis helpers ([`time_ticks`], [`time_hm`], [`date_ymd`], ...)
//! - optional `theme` cargo feature: `Theme` (built-in themes via `Theme::all`,
//!   `Theme::next`, `Theme::prev`); every widget then has `.theme(impl Into<Theme>)`
//! - high resolution: [`Marker`] (`Braille` 2×4 default, `HalfBlock`, `Block`,
//!   `Dot`, opt-in `Sextant` 2×3 which needs a font with the Unicode
//!   "Symbols for Legacy Computing" block), [`CandleStyle::Braille`],
//!   `.marker(..)` on line/bar/sparkline/radar/gauge/histogram/depth/waterfall
//!   charts and [`OverlapPolicy`] for [`Canvas`] colour conflicts
//! - optional `serde` cargo feature: `Serialize`/`Deserialize` on data types and
//!   (with `theme`) `Theme`
//!
//! All widgets implement [`ratatui::widgets::Widget`] (both owned and by reference)
//! and are drawn straight into the buffer.
//!
//! ```
//! # #[cfg(feature = "theme")] {
//! use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
//! use tui_chart::{Axis, LineChart, Series, Theme};
//!
//! let chart = LineChart::new(vec![Series::new("sin").data(
//!     (0..50).map(|i| (i as f64, (i as f64 / 5.0).sin())),
//! )])
//! .theme(Theme::nord())
//! .x_axis(Axis::new().title("t"))
//! .y_axis(Axis::new().grid(true));
//!
//! let area = Rect::new(0, 0, 60, 15);
//! let mut buf = Buffer::empty(area);
//! chart.render(area, &mut buf);
//! # }
//! ```
//!
//! ```
//! use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
//! use tui_chart::{Candle, CandlestickChart, sma, Overlay};
//! use ratatui::style::Color;
//!
//! let candles = vec![
//!     Candle::new(10.0, 12.0, 9.0, 11.0).volume(1200.0).label("Mon"),
//!     Candle::new(11.0, 13.0, 10.5, 10.8).volume(900.0).label("Tue"),
//! ];
//! let avg = sma(&candles, 2);
//! let chart = CandlestickChart::new(candles)
//!     .show_volume(true)
//!     .overlay(Overlay::new("SMA2", avg, Color::Yellow));
//! let area = Rect::new(0, 0, 40, 16);
//! let mut buf = Buffer::empty(area);
//! chart.render(area, &mut buf);
//! ```
#![warn(missing_docs)]

#[macro_use]
mod macros;
mod annotation;
mod axis;
mod bar;
mod boxplot;
mod candle;
mod canvas;
mod crosshair;
mod depth;
mod gantt;
mod gauge;
mod heatmap;
mod histogram;
mod indicator;
mod layout;
mod line;
mod radar;
mod renko;
mod scale;
mod sparkline;
mod stack;
mod stream;
mod text;
mod theme;
mod timeaxis;
mod treemap;
mod waterfall;

pub mod util;

pub use annotation::{draw_annotations, Annotation, AnnotationKind};
pub use axis::{Axis, GridStyle};
pub use bar::{BarChart, BarGroup, BarMode, BarOrientation, LabelPosition};
pub use boxplot::{BoxOrientation, BoxPlot, BoxStats};
pub use candle::{
    ema, heikin_ashi, sma, BandOverlay, Candle, CandleStyle, CandlestickChart, HLine,
    IndicatorPane, Overlay, TradeMarker, TradeSide,
};
pub use canvas::{Canvas, Marker, OverlapPolicy};
pub use crosshair::{nearest_point, Crosshair, NearestPoint};
pub use depth::DepthChart;
pub use gantt::{Gantt, Task};
pub use gauge::{Gauge, GaugeOrientation, GaugeStyle};
pub use heatmap::{Gradient, Heatmap};
pub use histogram::{Bin, Histogram};
pub use indicator::{atr, bollinger, macd, rsi, stochastic, vwap};
pub use line::{LegendPosition, LineChart, LineMode, Series};
pub use radar::{RadarChart, RadarSeries};
pub use renko::{renko_bricks, RenkoBrick, RenkoChart, RenkoStyle};
pub use sparkline::{Sparkline, SparklineStyle};
pub use stack::{shared_gutter, ChartStack, StackDirection};
pub use stream::{RingBuffer, Stream};
#[cfg(feature = "theme")]
pub use theme::{ChartThemeExt, Theme};
pub use timeaxis::{
    civil_from_days, date_md, date_ymd, datetime, days_from_civil, time_hm, time_hms,
    time_nice_step, time_ticks,
};
pub use treemap::{Treemap, TreemapItem};
#[cfg(feature = "theme")]
pub use tui_nuances::{Status, ThemeProvider, ThemeSwitcher, Themed};
pub use waterfall::Waterfall;

use ratatui::style::Color;

/// Default series color cycle.
pub const PALETTE: [Color; 8] = [
    Color::Cyan,
    Color::Yellow,
    Color::Magenta,
    Color::Green,
    Color::LightRed,
    Color::LightBlue,
    Color::LightGreen,
    Color::White,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_not_empty() {
        assert_eq!(PALETTE.len(), 8);
    }
}
