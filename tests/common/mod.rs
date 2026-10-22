//! Shared helpers for the matrix-style integration tests.
#![allow(dead_code)]

use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use tui_chart::*;

/// Kind of input data fed to every widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Data {
    Empty,
    Single,
    Nasty,
    Normal,
}

pub const DATAS: [Data; 4] = [Data::Empty, Data::Single, Data::Nasty, Data::Normal];

/// Areas used by the robustness matrix.
pub const AREAS: [(u16, u16); 8] = [
    (0, 0),
    (1, 1),
    (2, 2),
    (3, 1),
    (1, 40),
    (10, 3),
    (80, 24),
    (300, 80),
];

pub fn vals(d: Data) -> Vec<f64> {
    match d {
        Data::Empty => vec![],
        Data::Single => vec![5.0],
        Data::Nasty => vec![
            f64::NAN,
            f64::INFINITY,
            -1.0,
            1e300,
            f64::NEG_INFINITY,
            -1e300,
            0.0,
            f64::MAX,
            f64::MIN,
            3.0,
        ],
        Data::Normal => (0..30)
            .map(|i| ((i as f64) * 0.4).sin() * 10.0 + 12.0)
            .collect(),
    }
}

pub fn candles(d: Data) -> Vec<Candle> {
    match d {
        Data::Empty => vec![],
        Data::Single => vec![Candle::new(10.0, 12.0, 9.0, 11.0).volume(5.0)],
        Data::Nasty => vec![
            Candle::new(f64::NAN, f64::INFINITY, -1e300, 1e300),
            Candle::new(1.0, 1.0, 1.0, 1.0).volume(f64::NAN),
            Candle::new(-5.0, -2.0, -9.0, -3.0).volume(-4.0),
            Candle::new(f64::MAX, f64::MAX, f64::MIN, 0.0).volume(f64::INFINITY),
        ],
        Data::Normal => (0..40)
            .map(|i| {
                let b = 50.0 + (i as f64 * 0.3).sin() * 8.0;
                Candle::new(b, b + 4.0, b - 3.0, b + if i % 2 == 0 { 2.0 } else { -2.0 })
                    .volume(10.0 * (i + 1) as f64)
            })
            .collect(),
    }
}

fn pairs(d: Data) -> Vec<(f64, f64)> {
    vals(d)
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as f64, v))
        .collect()
}

/// A render closure that can be invoked repeatedly.
pub type Draw = Box<dyn Fn(Rect, &mut Buffer)>;

fn boxed<W>(w: W) -> Draw
where
    W: Fn() -> Box<dyn FnOnce(Rect, &mut Buffer)> + 'static,
{
    Box::new(move |a, b| w()(a, b))
}

fn mk<W: Widget + 'static>(f: impl Fn() -> W + 'static) -> Draw {
    boxed(move || {
        let w = f();
        Box::new(move |a, b| w.render(a, b))
    })
}

/// Theme type accepted by [`all_widgets`]; uninhabited without the `theme` feature.
#[cfg(feature = "theme")]
pub type Th = Theme;
#[cfg(not(feature = "theme"))]
#[derive(Clone)]
pub enum Th {}

/// Every public widget, built from `d`, optionally themed. Names are stable.
pub fn all_widgets(d: Data, theme: Option<Th>) -> Vec<(&'static str, Draw)> {
    let t = move || theme.clone();
    macro_rules! th {
        ($w:expr, $t:expr) => {{
            let w = $w;
            #[cfg(feature = "theme")]
            let w = match $t {
                Some(t) => w.theme(t),
                None => w,
            };
            #[cfg(not(feature = "theme"))]
            let _ = $t;
            w
        }};
    }
    let mut v: Vec<(&'static str, Draw)> = Vec::new();
    let c = t();
    v.push((
        "LineChart",
        mk(move || {
            th!(
                LineChart::new(vec![
                    Series::new("a").data(pairs(d)),
                    Series::new("b").values(vals(d)),
                ])
                .block(ratatui::widgets::Block::bordered().title("t")),
                c.clone()
            )
        }),
    ));
    let c = t();
    v.push((
        "BarChart",
        mk(move || {
            let groups = if d == Data::Empty {
                vec![]
            } else {
                vec![BarGroup::new("g1", vals(d)), BarGroup::new("g2", vals(d))]
            };
            th!(BarChart::new(groups), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "CandlestickChart",
        mk(move || {
            let cs = candles(d);
            th!(
                CandlestickChart::new(cs.clone())
                    .show_volume(true)
                    .overlay(Overlay::new("SMA", sma(&cs, 3), Color::Yellow))
                    .pane(IndicatorPane::Rsi { period: 5 }),
                c.clone()
            )
        }),
    ));
    let c = t();
    v.push((
        "Histogram",
        mk(move || th!(Histogram::new(vals(d)), c.clone())),
    ));
    let c = t();
    v.push((
        "Heatmap",
        mk(move || {
            let rows = match d {
                Data::Empty => vec![],
                _ => (0..4).map(|_| vals(d)).collect(),
            };
            th!(Heatmap::new(rows), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "Gauge",
        mk(move || {
            th!(
                Gauge::new(vals(d).first().copied().unwrap_or(f64::NAN)),
                c.clone()
            )
        }),
    ));
    let c = t();
    v.push((
        "RadarChart",
        mk(move || {
            let vs = vals(d);
            let axes: Vec<String> = (0..vs.len().max(3)).map(|i| format!("a{i}")).collect();
            let r = RadarChart::new(axes).series(RadarSeries::new("s", vs));
            th!(r, c.clone())
        }),
    ));
    let c = t();
    v.push((
        "Funnel",
        mk(move || {
            let st: Vec<FunnelStage> = vals(d)
                .into_iter()
                .enumerate()
                .map(|(i, x)| FunnelStage::new(format!("s{i}"), x))
                .collect();
            th!(Funnel::new(st), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "Bullet",
        mk(move || {
            let it: Vec<BulletItem> = vals(d)
                .into_iter()
                .enumerate()
                .map(|(i, x)| {
                    BulletItem::new(format!("b{i}"), x)
                        .target(x * 1.1)
                        .ranges([x, x * 2.0])
                })
                .collect();
            th!(Bullet::new(it), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "PairedBars",
        mk(move || {
            let it: Vec<PairedBarsItem> = vals(d)
                .into_iter()
                .enumerate()
                .map(|(i, x)| PairedBarsItem::new(format!("c{i}"), x, x * 0.5).unit("%"))
                .collect();
            th!(PairedBars::new(it).legend("a", "b"), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "BubbleChart",
        mk(move || {
            let pts: Vec<(f64, f64, f64)> = vals(d).into_iter().map(|x| (x, x, x)).collect();
            th!(BubbleChart::new([BubbleSeries::new("a", pts)]), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "LollipopChart",
        mk(move || {
            let it: Vec<LollipopItem> = vals(d)
                .into_iter()
                .enumerate()
                .map(|(i, x)| LollipopItem::new(format!("l{i}"), x).from(x * 0.5))
                .collect();
            th!(LollipopChart::new(it), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "SlopeChart",
        mk(move || {
            let it: Vec<SlopeItem> = vals(d)
                .into_iter()
                .enumerate()
                .map(|(i, x)| SlopeItem::new(format!("s{i}"), x, x * 1.5 - 2.0))
                .collect();
            th!(SlopeChart::new(it).titles("a", "b"), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "CalendarHeatmap",
        mk(move || th!(CalendarHeatmap::new(19_000, vals(d)), c.clone())),
    ));
    let c = t();
    v.push((
        "ViolinChart",
        mk(move || {
            th!(
                ViolinChart::new([
                    ViolinGroup::new("a", vals(d)),
                    ViolinGroup::new("b", vals(d))
                ]),
                c.clone()
            )
        }),
    ));
    let c = t();
    v.push((
        "RoseChart",
        mk(move || {
            let it: Vec<RoseItem> = vals(d)
                .into_iter()
                .enumerate()
                .map(|(i, x)| RoseItem::new(format!("r{i}"), x))
                .collect();
            th!(RoseChart::new(it), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "StackedAreaChart",
        mk(move || {
            th!(
                StackedAreaChart::new([
                    AreaLayer::new("a", vals(d)),
                    AreaLayer::new("b", vals(d).into_iter().rev())
                ]),
                c.clone()
            )
        }),
    ));
    let c = t();
    v.push((
        "Sankey",
        mk(move || {
            let l: Vec<SankeyLink> = vals(d)
                .into_iter()
                .enumerate()
                .map(|(i, x)| {
                    SankeyLink::new(format!("n{}", i % 4), format!("n{}", (i + 1) % 5), x)
                })
                .collect();
            th!(Sankey::new(l), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "Treemap",
        mk(move || {
            let items: Vec<TreemapItem> = vals(d)
                .into_iter()
                .enumerate()
                .map(|(i, x)| TreemapItem::new(format!("i{i}"), x))
                .collect();
            th!(Treemap::new(items), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "BoxPlot",
        mk(move || {
            let mut b = BoxPlot::new();
            if let Some(s) = BoxStats::from_samples(&vals(d), 1.5) {
                b = b.stats("x", s);
            }
            if d == Data::Nasty {
                b = b.stats(
                    "n",
                    BoxStats::new(f64::NAN, 1.0, f64::INFINITY, -1.0, 1e300),
                );
            }
            th!(b, c.clone())
        }),
    ));
    let c = t();
    v.push((
        "Waterfall",
        mk(move || {
            let mut w = Waterfall::new();
            for (i, x) in vals(d).into_iter().enumerate() {
                w = w.delta(format!("s{i}"), x);
            }
            if d != Data::Empty {
                w = w.total("T");
            }
            th!(w, c.clone())
        }),
    ));
    let c = t();
    v.push((
        "Gantt",
        mk(move || {
            let tasks: Vec<Task> = vals(d)
                .into_iter()
                .enumerate()
                .map(|(i, x)| Task::new(format!("t{i}"), x, x + 3.0).progress(0.5))
                .collect();
            th!(Gantt::new(tasks), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "DepthChart",
        mk(move || {
            let vs = vals(d);
            let bids: Vec<(f64, f64)> = vs.iter().map(|x| (*x, x.abs() + 1.0)).collect();
            let asks: Vec<(f64, f64)> = vs.iter().map(|x| (x + 20.0, x.abs() + 2.0)).collect();
            th!(DepthChart::new(bids, asks), c.clone())
        }),
    ));
    let c = t();
    v.push((
        "RenkoChart",
        mk(move || th!(RenkoChart::new(&vals(d), 1.0), c.clone())),
    ));
    let c = t();
    v.push((
        "RenkoChart(zero brick)",
        mk(move || th!(RenkoChart::new(&vals(d), 0.0), c.clone())),
    ));
    let c = t();
    v.push((
        "Sparkline",
        mk(move || th!(Sparkline::new(vals(d)), c.clone())),
    ));
    v
}

/// Render `draw` into a buffer that is larger than `area`, with a sentinel
/// symbol everywhere; return the buffer.
pub fn render_padded(draw: &Draw, area: Rect) -> (Buffer, Rect) {
    let full = Rect::new(0, 0, area.x + area.width + 3, area.y + area.height + 3);
    let mut buf = Buffer::empty(full);
    for c in buf.content.iter_mut() {
        c.set_symbol("§");
    }
    draw(area, &mut buf);
    (buf, full)
}

/// Assert every cell outside `area` still holds the sentinel.
pub fn assert_no_outside_writes(name: &str, buf: &Buffer, full: Rect, area: Rect) {
    for y in full.y..full.bottom() {
        for x in full.x..full.right() {
            let inside = x >= area.x && x < area.right() && y >= area.y && y < area.bottom();
            if !inside {
                assert_eq!(
                    buf[(x, y)].symbol(),
                    "§",
                    "{name}: write outside area {area:?} at ({x},{y})"
                );
            }
        }
    }
}

pub fn rows(buf: &Buffer, area: Rect) -> Vec<String> {
    (area.y..area.bottom())
        .map(|y| {
            (area.x..area.right())
                .map(|x| buf[(x, y)].symbol())
                .collect()
        })
        .collect()
}
