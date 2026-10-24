# tui-chart

[![CI](https://github.com/sorinirimies/tui-chart/actions/workflows/ci.yml/badge.svg)](https://github.com/sorinirimies/tui-chart/actions)
[![Security](https://github.com/sorinirimies/tui-chart/actions/workflows/security.yml/badge.svg)](https://github.com/sorinirimies/tui-chart/actions/workflows/security.yml)
[![Fuzz](https://github.com/sorinirimies/tui-chart/actions/workflows/fuzz.yml/badge.svg)](https://github.com/sorinirimies/tui-chart/actions/workflows/fuzz.yml)
[![Crates.io](https://img.shields.io/crates/v/tui-chart?style=flat-square&logo=rust&color=orange)](https://crates.io/crates/tui-chart)
[![Downloads](https://img.shields.io/crates/d/tui-chart)](https://crates.io/crates/tui-chart)
[![Documentation](https://docs.rs/tui-chart/badge.svg)](https://docs.rs/tui-chart)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

![tui-chart demo](examples/vhs/output/tui_chart.gif)

Terminal chart widgets for [ratatui](https://ratatui.rs). Pie charts live in
[tui-piechart](https://github.com/sorinirimies/tui-piechart).

## Chart types

| Widget | Description |
|---|---|
| `LineChart` | line, step, area, scatter, multi-series, legend, secondary y axis |
| `CandlestickChart` | candles, Heikin-Ashi, OHLC, volume, overlays, indicator panes, trade markers |
| `BarChart` | vertical/horizontal, grouped/stacked |
| `Histogram` | auto-binned distribution, KDE overlay, mean/median markers (`.stats`) |
| `Heatmap` | multiple color gradients, `.stretch(true)` to fill the area |
| `Sparkline` | bars, area, line, win/loss, moving-average `.smooth(n)` |
| `Gauge` | 16 styles: linear, bullet, segmented, arc, dial, meter, reticle, ring, chevron, smooth, dots, LED, battery, level, thermometer, signal; bands, target, scale |
| `RadarChart` | multi-axis paired_bars, filled polygons, ring values, legend |
| `BoxPlot` | quartiles, whiskers, outliers, mean marker, notches, jittered points |
| `Waterfall` | running deltas and totals |
| `Gantt` | task timeline with progress |
| `DepthChart` | order book bids/asks |
| `RenkoChart` | fixed-size price bricks |
| `Treemap` | nested weighted rectangles with group headers and tile gutters |
| `RadialBar` / `RingStack` | concentric progress rings (270° or full), legend with percentages |
| `Oscilloscope` / `Spectrum` / `DotMatrix` / `RadarScope` / `HexMap` | instrument panels: ECG/wave trace, equalizer with peak caps, LED dot matrix, sweeping radar scope, honeycomb heat map (animate with `.phase(t)`) |
| `Fx` / `glow` / `scanlines` / `chamfer_block` / `bracket_block` | cyberpunk post effects for any widget: glow, scanlines, cut-corner frames |
| `HitMap` / `Interaction` | mouse interactivity for every chart: hover tooltips, click to pin, click a legend entry to hide a series |
| `Funnel` | tapered stages, side labels, conversion % and drop-off (`.taper`, `.show_dropoff`) |
| `Bullet` | value vs target over qualitative ranges |
| `PairedBars` | before/after bar pairs, each measure on its own scale, inline or stacked, with a legend |
| `BubbleChart` | scatter with area-scaled bubble sizes |
| `LollipopChart` | lollipop stems and dumbbells (`LollipopItem::from`), sortable |
| `SlopeChart` | two-state paired_bars with trend colors and non-overlapping labels |
| `CalendarHeatmap` | GitHub-style daily calendar with levels, month/weekday labels, legend, missing/zero handling |
| `ViolinChart` | mirrored KDE per group with quartile box and median |
| `RoseChart` | polar-area / Nightingale chart, area or linear scale, optional hole |
| `StackedAreaChart` | stacked or 100 % normalised areas with legend |
| `Sankey` | flow diagram with auto layout and weighted ribbons |
| `ChartStack` | vertical/horizontal stack with shared gutter |

## Gallery

### LineChart

![LineChart](examples/vhs/output/line.gif)

### CandlestickChart

![CandlestickChart](examples/vhs/output/candlestick.gif)

### BarChart

![BarChart](examples/vhs/output/bars.gif)

### Histogram

![Histogram](examples/vhs/output/histogram.gif)

### Heatmap

![Heatmap](examples/vhs/output/heatmap.gif)

### Sparkline

![Sparkline](examples/vhs/output/sparkline.gif)

### Gauge

![Gauge](examples/vhs/output/gauge.gif)

### RadialBar

![RadialBar](examples/vhs/output/radial.gif)

### Instruments

![Instruments](examples/vhs/output/instruments.gif)

### Cyber

![Cyber](examples/vhs/output/cyber.gif)

### Interactive

![Interactive](examples/vhs/output/interactive.gif)

### RadarChart

![RadarChart](examples/vhs/output/radar.gif)

### BoxPlot

![BoxPlot](examples/vhs/output/boxplot.gif)

### Waterfall

![Waterfall](examples/vhs/output/waterfall.gif)

### Gantt

![Gantt](examples/vhs/output/gantt.gif)

### DepthChart

![DepthChart](examples/vhs/output/depth.gif)

### RenkoChart

![RenkoChart](examples/vhs/output/renko.gif)

### Treemap

![Treemap](examples/vhs/output/treemap.gif)

### ChartStack

![ChartStack](examples/vhs/output/chartstack.gif)

### Funnel

![Funnel](examples/vhs/output/funnel.gif)

### Bullet

![Bullet](examples/vhs/output/bullet.gif)

### PairedBars

![PairedBars](examples/vhs/output/paired_bars.gif)

### Bubble

![Bubble](examples/vhs/output/bubble.gif)

### LollipopChart

![LollipopChart](examples/vhs/output/lollipop.gif)

### SlopeChart

![SlopeChart](examples/vhs/output/slope.gif)

### CalendarHeatmap

![CalendarHeatmap](examples/vhs/output/calendar.gif)

- `NaN` days draw as a dim `·` (missing); `±inf` clamp to the top / bottom of the range.
- `zero_empty(true)` draws exact-zero days in the dim theme colour, distinct from the lowest level.
- Flat data renders at the lowest level (all zero) or mid-scale instead of max colour; `bounds()` swaps an inverted range.
- At height 7 the labels are dropped so the grid still draws; the legend needs height 10+ and shows a 9-step ramp in `continuous()` mode.

### PieChart

![PieChart](examples/vhs/output/pie.gif)

Pie charts come from the companion crate [tui-piechart](https://github.com/sorinirimies/tui-piechart),
re-exported here (default `piechart` feature) so a single `tui-chart` dependency gives you both:

```rust
use tui_chart::{PieChart, PieSlice}; // or tui_chart::piechart::*
```

Disable with `default-features = false` if you don't need it; `theme` forwards to `tui-piechart/theme`.

### ViolinChart

![ViolinChart](examples/vhs/output/violin.gif)

### RoseChart

![RoseChart](examples/vhs/output/rose.gif)

### StackedAreaChart

![StackedAreaChart](examples/vhs/output/stacked_area.gif)

### Sankey

![Sankey](examples/vhs/output/sankey.gif)

### Live streaming

![Live streaming](examples/vhs/output/stream.gif)

## High resolution (braille)

Dot-based drawing is selected with `Marker`; braille (2×4 dots per cell) is the default.

| Marker | Dots / cell | Glyphs | Notes |
|---|---|---|---|
| `Braille` | 2×4 | `U+2800..U+28FF` | default, widely supported |
| `HalfBlock` | 1×2 | `▀ ▄ █` | |
| `Block` | 1×1 | `█` | |
| `Dot` | 1×1 | `•` | |
| `Sextant` | 2×3 | `U+1FB00..U+1FB3B` (+ `▌ ▐ █`) | opt-in: needs a font with "Symbols for Legacy Computing"; otherwise renders as tofu/blank |

- `CandleStyle::Braille`: 1-dot wicks, 2-dot-wide bodies, ~2× candles per width; works with bull/bear colors, themes, overlays, volume and panes.
- `BarChart::marker(m)` / `Histogram::bar_marker(m)`: bars built from dots (4 vertical dots per cell on braille); default stays eighth blocks.
- `.marker(m)` on `LineChart`, `Sparkline` (line style), `RadarChart`, `Gauge` (arc), `Histogram` (KDE), `DepthChart` (step-area), `Waterfall` (connectors; default `┄`).
- `Canvas::set_overlap(OverlapPolicy::{LastWins, FirstWins})`: which colour a cell takes when dots of several colours share it (default `LastWins`).
- Demos: press `k` to cycle markers (`s` cycles candle styles incl. braille).

## Features

- Annotations (`Annotation`: hline, vline, regions, labels) and `Crosshair`
- Log scale (`y_log`), spline smoothing, error bars and bands
- Indicators: `sma`, `ema`, `bollinger`, `rsi`, `macd`, `atr`, `stochastic`, `vwap`
- Streaming data with `RingBuffer` / `Stream`
- Time axis helpers (`time_ticks`, `time_hm`, `date_ymd`, ...)
- `ChartStack` with shared gutter
- Optional theming via the `theme` cargo feature (below)

## Theming

Theming is opt-in. Enable the `theme` cargo feature (it pulls in `tui-nuances`):

```toml
tui-chart = { version = "0.1", features = ["theme"] }
```

Without it the crate has no `tui-nuances` dependency and every widget uses its built-in colors.

`tui_chart::Theme` is the shared [`tui-nuances`](https://github.com/sorinirimies/tui-nuances)
theme (re-exported), so every component by the same author (`tui-slider`, `tui-spinner`, ...)
can be styled from one `Theme`. Every widget accepts `.theme(impl Into<Theme>)` (and implements
`tui_nuances::Themed`, so `.apply_theme(&theme)` / `.themed(&provider)` work too).

```rust
use tui_chart::{LineChart, Theme};

let theme = Theme::tokyo_night();
let chart = LineChart::new(series).theme(&theme);
let next = theme.next(); // cycle with next()/prev(), or use ThemeSwitcher
```

Built-in themes (`Theme::all()`): `default`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`,
`nord`, `gruvbox`, `tokyo-night`, `solarized-dark`, `solarized-light`, `one-dark`, `rose-pine`,
`monokai`, `porsche`, `colorblind`, `high-contrast`, `mono`.

Chart-specific derived values (heatmap `gradient()`, crosshair/tooltip styles, palette cycling)
come from the `ChartThemeExt` trait.

**Precedence:** explicit colors/styles (series colors, candle colors, axis styles,
`.base_style(..)`) always beat the theme, whether set before or after `.theme(..)`.
With no theme set, rendering is unchanged.

### Using your app's theme

`Theme::from_base(name, bg, fg, accent, bull, bear)` derives the rest. Implement
`ThemeProvider` (any widget: `.themed(&app)`) or `From<&AppTheme> for Theme` once in your app:

```rust
impl From<&AppTheme> for tui_chart::Theme {
    fn from(a: &AppTheme) -> Self {
        tui_chart::Theme::from_base("app", Some(a.bg), a.text, a.brand, a.ok, a.err)
    }
}
let chart = LineChart::new(series).theme(&app_theme);
```

## Accessibility

Wrap any chart in `Monochrome` (or `Monochrome::auto`, which honours the
[`NO_COLOR`](https://no-color.org) convention) to render without colour. Solid fills
(heatmap, treemap, calendar, bars) become `░▒▓█` shades by intensity, so values stay readable:

```rust
use tui_chart::{Heatmap, Monochrome};
frame.render_widget(Monochrome::auto(Heatmap::new(data)), area);
```

## Cargo features

| Feature | Description |
|---|---|
| `theme` | `Theme`, `.theme(..)` on every widget, `Themed`/`ThemeProvider`/`ThemeSwitcher` re-exports (adds the `tui-nuances` dependency; off by default) |
| `serde` | `Serialize`/`Deserialize` for data types (and `Theme` with `theme`) (`Candle`, `BarGroup`, `Bin`, `BoxStats`, `Task`, `Annotation`, `TradeMarker`, ...) |

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
# examples need the theme feature: cargo run --features theme --example <name>
cargo run --features theme --example bars
cargo run --features theme --example boxplot
cargo run --features theme --example candlestick
cargo run --features theme --example bubble
cargo run --features theme --example lollipop
cargo run --features theme --example slope
cargo run --features theme --example calendar
cargo run --example pie
cargo run --features theme --example violin
cargo run --features theme --example rose
cargo run --features theme --example stacked_area
cargo run --features theme --example sankey
cargo run --features theme --example bullet
cargo run --features theme --example paired_bars
cargo run --features theme --example funnel
cargo run --features theme --example interactive
cargo run --features theme --example cyber
cargo run --features theme --example instruments
cargo run --features theme --example radial
cargo run --features theme --example chartstack
cargo run --features theme --example depth
cargo run --features theme --example gantt
cargo run --features theme --example gauge
cargo run --features theme --example heatmap
cargo run --features theme --example histogram
cargo run --features theme --example line
cargo run --features theme --example radar
cargo run --features theme --example renko
cargo run --features theme --example showcase
cargo run --features theme --example sparkline
cargo run --features theme --example stream
cargo run --features theme --example themes
cargo run --features theme --example treemap
cargo run --features theme --example waterfall
```

Render timing: `cargo run --release --example bench`. Or `just example-<name>` / `just examples`. Demo GIFs: `just vhs-all` (or `just vhs-<name>`) (needs [vhs](https://github.com/charmbracelet/vhs) and nushell).

## Development

```sh
just check   # fmt, clippy, tests
```

Scripts in `scripts/` are nushell and have their own tests (`just test-nu`).
See [CONTRIBUTING.md](CONTRIBUTING.md).

### Security & fuzzing

Dependencies are checked daily with `cargo audit` and `cargo deny` (the
Security workflow), and every widget renderer, the indicators, the time-axis
helpers and the serde path are fuzzed with `cargo-fuzz` (`fuzz/`, the Fuzz
workflow). Locally: `just audit`, `just deny`, `just fuzz render_line`,
`just fuzz-all`.

## Interactivity

Charts are stateless widgets, so interactivity is a small loop in your app (see
`examples/interactive.rs`). It works on every chart and with any backend; it only
needs cell coordinates.

```rust
let mut ui = Interaction::default();          // keep between frames
let mut hits = HitMap::default();

// draw: render through `capture` so the chart reports what it drew
hits = HitMap::capture(LineChart::new(series).hidden(ui.hidden()), area, frame.buffer_mut());
ui.render_tooltip(&hits, frame.buffer_mut(), area);

// events
ui.on_move(mouse.column, mouse.row);          // hover  -> tooltip + highlight
ui.on_click(mouse.column, mouse.row, &hits);  // click  -> pin, or toggle a legend entry
```

- **Hover tooltips** for points, bars, candles, slices, cells, tasks, nodes ... on every chart
  (`Hit` carries a title and value lines, so you can also render your own).
- **Click** pins a tooltip; clicking again clears it.
- **Legend toggling**: clicking a legend entry hides that series. Pass `ui.hidden()` to
  `.hidden(..)` on `LineChart`, `BarChart`, `StackedAreaChart` and `RadarChart`; the axes rescale.
- `HitMap::at`, `nearest`, `pick` and `legend_at` give raw hit-testing for custom behaviour.
- `PieChart` lives in `tui-piechart` and does not report hits.

## Cyberpunk effects

`Fx`, `glow`, `scanlines`, `chamfer_block` and `bracket_block` work on the rendered buffer, so no GPU is needed.

```rust
Fx::new(Spectrum::new(bars).block(chamfer_block(" EQ "))).glow(1, 0.25).scanlines(0.3)
```

`cargo run --release --features theme --example cyber`

## License

MIT
