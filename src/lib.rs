//! # tui-chart
//!
//! Chart widgets for [ratatui](https://ratatui.rs): line / step / area / scatter,
//! bar (vertical, horizontal, grouped, stacked), candlestick & OHLC (with volume
//! and indicator overlays), histogram and heatmap.
//!
//! All widgets implement [`ratatui::widgets::Widget`] (both owned and by reference)
//! and are drawn straight into the buffer; no extra runtime dependencies.
//!
//! ```
//! use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
//! use tui_chart::{Axis, LineChart, Series};
//!
//! let chart = LineChart::new(vec![Series::new("sin").data(
//!     (0..50).map(|i| (i as f64, (i as f64 / 5.0).sin())),
//! )])
//! .x_axis(Axis::new().title("t"))
//! .y_axis(Axis::new().grid(true));
//!
//! let area = Rect::new(0, 0, 60, 15);
//! let mut buf = Buffer::empty(area);
//! chart.render(area, &mut buf);
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

mod axis;
mod bar;
mod candle;
mod canvas;
mod heatmap;
mod histogram;
mod layout;
mod line;
mod sparkline;
mod gauge;
mod radar;
mod boxplot;
mod waterfall;
mod gantt;
mod depth;
mod renko;
mod treemap;

pub mod util;

pub use axis::Axis;
pub use bar::{BarChart, BarGroup, BarMode, BarOrientation};
pub use candle::{ema, heikin_ashi, sma, Candle, CandleStyle, CandlestickChart, Overlay};
pub use canvas::{Canvas, Marker};
pub use heatmap::{Gradient, Heatmap};
pub use histogram::{Bin, Histogram};
pub use line::{LegendPosition, LineChart, LineMode, Series};
pub use sparkline::Sparkline;
pub use gauge::Gauge;
pub use radar::RadarChart;
pub use boxplot::BoxPlot;
pub use waterfall::Waterfall;
pub use gantt::Gantt;
pub use depth::DepthChart;
pub use renko::RenkoChart;
pub use treemap::Treemap;

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
