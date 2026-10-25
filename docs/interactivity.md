# Interactivity guide

tui-chart widgets are stateless: they draw what you give them. Interactivity is a
small loop in *your* app built from three types:

| Type | Role |
|------|------|
| [`HitMap`] | what a chart drew, per frame: points, bars, slices, legend entries |
| [`Interaction`] | pointer state: hover, pinned selection, hidden series |
| [`View`] | zoom / pan window for the x axis |

They only deal in terminal **cell coordinates**, so they work with any backend
(crossterm, termion, termwiz, a web terminal) and any event source.

## 1. The loop

```rust
use crossterm::event::{self, Event, MouseButton, MouseEventKind, EnableMouseCapture, DisableMouseCapture};
use tui_chart::{HitMap, Interaction, LineChart, View};

let mut ui = Interaction::default();
let mut view = View::default();
let mut hits = HitMap::default();
let mut plot = ratatui::layout::Rect::default();

crossterm::execute!(std::io::stdout(), EnableMouseCapture)?;
loop {
    terminal.draw(|f| {
        plot = f.area();
        let (x0, x1) = view.range(0.0, 99.0);              // zoom window -> axis bounds
        let chart = LineChart::new(series.clone())
            .x_axis(Axis::new().bounds(x0, x1))
            .hidden(ui.hidden());                          // legend toggles
        hits = HitMap::capture(chart, plot, f.buffer_mut()); // draw + record
        ui.render_tooltip(&hits, f.buffer_mut(), plot);    // hover / pinned tooltip
    })?;

    if let Event::Mouse(m) = event::read()? {
        match m.kind {
            MouseEventKind::Moved => ui.on_move(m.column, m.row),
            MouseEventKind::Down(MouseButton::Left) => { ui.on_click(m.column, m.row, &hits); }
            MouseEventKind::ScrollUp => view.on_scroll(m.column, plot, 1),
            MouseEventKind::ScrollDown => view.on_scroll(m.column, plot, -1),
            _ => {}
        }
    }
}
crossterm::execute!(std::io::stdout(), DisableMouseCapture)?;
```

Keep `ui`, `view` and the latest `hits` between frames. `HitMap::capture`
renders the widget as normal into the frame buffer, so there is no extra cost
other than collecting the hit rectangles.

## 2. What you get

* **Hover tooltip**: `ui.render_tooltip` draws a box next to the pointer for the
  element under it (or the nearest one within two cells) and keeps it inside
  `bounds`.
* **Click to pin**: `on_click` returns a `Click`: `Selected`, `Cleared`,
  `Toggled { series, hidden }` (legend entry) or `Nothing`.
* **Legend toggling**: pass `ui.hidden()` to `.hidden(..)` of the chart.
  Hidden series are dropped from the data (axes rescale where the chart
  autoscales) and stay in the legend, struck through.
* **Zoom and pan**: `View` holds a `lo..hi` window of the full x span.
  `view.range(min, max)` maps it to axis bounds, `view.slice(&data)` cuts a
  slice for candle, bar or histogram data. Use `view.pan(±0.25)` for `[` / `]`.

### Charts and what they report

| Report points | Legend toggling (`.hidden`) |
|---------------|-----------------------------|
| every chart: line, bar, histogram, sparkline, candle, depth, renko, bubble, violin, boxplot, heatmap, calendar, gantt, sankey, slope, lollipop, paired bars, waterfall, funnel, treemap, bullet, rose, radial, radar, gauge, stacked area, **pie** (via [`Pie`]) and the instruments (oscilloscope, spectrum, dot matrix, radar scope, hex map) | `LineChart`, `BarChart`, `StackedAreaChart`, `RadarChart`, `RadialBar`, `RingStack`, `Funnel`, `Pie` |

## 3. Pie charts

`PieChart` is defined in `tui-piechart`, so wrap it:

```rust
use tui_chart::{Pie, PieChart, PieSlice};
let chart = PieChart::new(slices).hole(0.4);
hits = HitMap::capture(Pie::new(chart).hidden(ui.hidden()), area, buf);
```

All styles (`Solid`, `Segmented`, `Hud`, `Rose`), donuts and resolutions work.

## 4. Custom behaviour

`HitMap` is plain data. Use `hits.at(x, y)`, `nearest`, `pick` or
`legend_at` to build your own: a status line, a detail pane, a drill-down. A
`Hit` carries `kind`, `series`, `index`, `title`, `lines` and its `rect`.

## 5. No mouse?

Everything takes cell coordinates, so drive it from the keyboard by moving a
virtual pointer: arrows call `ui.on_move(x ± 1, y ± 1)`, `space` calls
`ui.on_click(x, y, &hits)`, `+` / `-` call `view.zoom(..)`. See
`examples/interactive.rs` (the VHS demo is recorded this way).

## 6. Embedding in other layouts

Charts inside `ChartStack`, tabs or popups just work: capture the *whole* frame
region you draw into, or call `HitMap::capture` per chart and keep the map of the
chart under the pointer. Hit rectangles are in absolute buffer coordinates.
