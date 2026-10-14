# tui-chart

[![CI](https://github.com/sorinirimies/tui-chart/actions/workflows/ci.yml/badge.svg)](https://github.com/sorinirimies/tui-chart/actions)
[![Crates.io](https://img.shields.io/crates/v/tui-chart.svg)](https://crates.io/crates/tui-chart)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Terminal chart widgets for [ratatui](https://ratatui.rs). Pie charts live in
[tui-piechart](https://github.com/sorinirimies/tui-piechart).

## Chart types

| Widget | Description |
|---|---|
| `LineChart` | line, step, area, scatter, multi-series, legend, secondary y axis |
| `CandlestickChart` | candles, Heikin-Ashi, OHLC, volume, overlays, indicator panes, trade markers |
| `BarChart` | vertical/horizontal, grouped/stacked |
| `Histogram` | auto-binned distribution |
| `Heatmap` | multiple color gradients |
| `Sparkline` | compact inline trend |
| `Gauge` | progress / meter |
| `RadarChart` | multi-axis comparison |
| `BoxPlot` | quartiles, whiskers, outliers |
| `Waterfall` | running deltas and totals |
| `Gantt` | task timeline with progress |
| `DepthChart` | order book bids/asks |
| `RenkoChart` | fixed-size price bricks |
| `Treemap` | nested weighted rectangles |
| `ChartStack` | vertical/horizontal stack with shared gutter |

Markers: `Braille`, `HalfBlock`, `Block`, `Dot` via `Marker`.

## Features

- Annotations (`Annotation`: hline, vline, regions, labels) and `Crosshair`
- Log scale (`y_log`), spline smoothing, error bars and bands
- Indicators: `sma`, `ema`, `bollinger`, `rsi`, `macd`, `atr`, `stochastic`, `vwap`
- Streaming data with `RingBuffer` / `Stream`
- Time axis helpers (`time_ticks`, `time_hm`, `date_ymd`, ...)
- `ChartStack` with shared gutter
- Themes (below)

## Theming

Every widget accepts `.theme(impl Into<Theme>)`.

```rust
use tui_chart::{LineChart, Theme};

let theme = Theme::tokyo_night();
let chart = LineChart::new(series).theme(theme.clone());
let next = theme.next(); // cycle with next()/prev()
```

Built-in themes (`Theme::all()`): `default`, `catppuccin_mocha`, `catppuccin_latte`,
`dracula`, `nord`, `gruvbox`, `tokyo_night`, `solarized_dark`, `porsche`, `colorblind`.

## Cargo features

| Feature | Description |
|---|---|
| `serde` | `Serialize`/`Deserialize` for `Theme` and data types (`Candle`, `BarGroup`, `Bin`, `BoxStats`, `Task`, `Annotation`, `TradeMarker`, ...) |

## Usage

```toml
[dependencies]
tui-chart = "0.1"
```

```rust
use ratatui::style::Color;
use tui_chart::{Axis, LineChart, Series};

let chart = LineChart::new(vec![
    Series::new("sin").data((0..100).map(|i| (i as f64, (i as f64 / 8.0).sin()))).color(Color::Cyan),
])
.x_axis(Axis::new().title("t"));
frame.render_widget(chart, area);
```

## Examples

```sh
cargo run --example bars
cargo run --example boxplot
cargo run --example candlestick
cargo run --example depth
cargo run --example gantt
cargo run --example gauge
cargo run --example heatmap
cargo run --example histogram
cargo run --example line
cargo run --example radar
cargo run --example renko
cargo run --example showcase
cargo run --example sparkline
cargo run --example stream
cargo run --example themes
cargo run --example treemap
cargo run --example waterfall
```

Or `just example-<name>` / `just examples`. Demo GIFs: `just vhs-all` (or `just vhs-<name>`) (needs [vhs](https://github.com/charmbracelet/vhs) and nushell).

## Development

```sh
just check   # fmt, clippy, tests
```

Scripts in `scripts/` are nushell. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
