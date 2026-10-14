# tui-chart

[![CI](https://github.com/sorinirimies/tui-chart/actions/workflows/ci.yml/badge.svg)](https://github.com/sorinirimies/tui-chart/actions)
[![Crates.io](https://img.shields.io/crates/v/tui-chart.svg)](https://crates.io/crates/tui-chart)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Terminal chart widgets for [ratatui](https://ratatui.rs): line, step, area, scatter, bar,
candlestick (stock/trading), histogram and heatmap. Pie charts live in
[tui-piechart](https://github.com/sorinirimies/tui-piechart).

## Chart types

| Widget | Styles |
|---|---|
| `LineChart` | `Line`, `Step`, `Area`, `Scatter`, multi-series, legend |
| `CandlestickChart` | `Candles`, `HeikinAshi`, `Ohlc`, SMA/EMA overlays, volume |
| `BarChart` | vertical/horizontal, grouped/stacked |
| `Histogram` | auto-binned distribution |
| `Heatmap` | multiple color gradients |

Markers: `Braille`, `HalfBlock`, `Block`, `Dot` via `Marker`.

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
cargo run --example line
cargo run --example candlestick
cargo run --example bars
cargo run --example histogram
cargo run --example heatmap
cargo run --example showcase
```

Demo GIFs: `just vhs-all` (needs [vhs](https://github.com/charmbracelet/vhs) and nushell).

## Development

```sh
just check   # fmt, clippy, tests
```

Scripts in `scripts/` are nushell. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
