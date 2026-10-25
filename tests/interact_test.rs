//! Every chart reports hit regions while rendering through `HitMap::capture`.

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_chart::*;

fn hits(w: impl Widget) -> HitMap {
    let area = Rect::new(0, 0, 60, 20);
    let mut buf = Buffer::empty(area);
    HitMap::capture(w, area, &mut buf)
}

fn points(h: &HitMap) -> usize {
    h.hits().iter().filter(|x| x.kind == HitKind::Point).count()
}

macro_rules! reports_hits {
    ($($name:ident: $chart:expr),* $(,)?) => {$(
        #[test]
        fn $name() {
            let h = hits($chart);
            assert!(points(&h) > 0, stringify!($name));
            // every hit lies inside the area and is addressable
            for hit in h.hits() {
                assert!(hit.rect.right() <= 60 && hit.rect.bottom() <= 20, "{hit:?}");
                let (x, y) = (hit.rect.x, hit.rect.y);
                assert!(h.hits().iter().any(|o| o.rect.x <= x && x < o.rect.right() && o.rect.y <= y && y < o.rect.bottom()));
            }
            // picking on a hit's own cell finds some element
            let first = h.hits().iter().find(|x| x.kind == HitKind::Point).unwrap();
            let c = first.shape.first().copied().unwrap_or(first.rect);
            assert!(h.pick(c.x, c.y).is_some());
        }
    )*};
}

reports_hits! {
    line: LineChart::new(vec![Series::new("a").data([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)])]),
    bar: BarChart::new(vec![BarGroup::new("a", [3.0]), BarGroup::new("b", [5.0])]),
    sparkline: Sparkline::new([1.0, 4.0, 2.0, 8.0, 5.0, 7.0]),
    histogram: Histogram::new((0..100).map(|i| ((i * 37) % 23) as f64)),
    candle: CandlestickChart::new(vec![Candle::new(10.0, 12.0, 9.0, 11.0), Candle::new(11.0, 13.0, 10.0, 12.5)]),
    depth: DepthChart::new([(99.0, 5.0), (98.0, 9.0)], [(101.0, 4.0), (102.0, 8.0)]),
    renko: RenkoChart::new(&[10.0, 12.0, 15.0, 13.0, 9.0], 1.0),
    bubble: BubbleChart::new([BubbleSeries::new("a", [(1.0, 2.0, 5.0), (3.0, 1.0, 9.0)])]),
    violin: ViolinChart::new([ViolinGroup::new("a", (0..60).map(|i| (i % 13) as f64))]),
    boxplot: BoxPlot::new().group("a", (1..=20).map(|x| x as f64)).group("b", (5..=30).map(|x| x as f64)),
    heatmap: Heatmap::new(vec![vec![0.0, 0.5, 1.0], vec![1.0, 0.5, 0.0]]),
    calendar: CalendarHeatmap::new(days_from_civil(2024, 1, 1), (0..120).map(|i| (i % 7) as f64)),
    gantt: Gantt::new([Task::new("Design", 0.0, 4.0), Task::new("Build", 3.0, 9.0)]),
    sankey: Sankey::new([SankeyLink::new("In", "Mid", 10.0), SankeyLink::new("Mid", "Out", 7.0)]),
    slope: SlopeChart::new([SlopeItem::new("a", 1.0, 3.0), SlopeItem::new("b", 4.0, 2.0)]).titles("2023", "2024"),
    lollipop: LollipopChart::new([LollipopItem::new("a", 3.0), LollipopItem::new("b", 5.0).from(1.0)]),
    waterfall: Waterfall::new().delta("Sales", 50.0).delta("Costs", -20.0).total("Net"),
    funnel: Funnel::new([FunnelStage::new("Visits", 1000.0), FunnelStage::new("Signups", 240.0)]),
    treemap: Treemap::new([TreemapItem::new("a", 5.0), TreemapItem::new("b", 3.0)]),
    pie: Pie::new(PieChart::new(vec![PieSlice::new("a", 3.0, ratatui::style::Color::Red), PieSlice::new("b", 5.0, ratatui::style::Color::Blue)])),
    oscilloscope: Oscilloscope::ecg(2, 120),
    spectrum: Spectrum::new([0.2, 0.8, 0.5]).peaks([0.3, 0.9, 0.6]),
    dot_matrix: DotMatrix::new([0.1, 0.4, 0.9]).rows(8),
    radar_scope: RadarScope::new([Blip::new(0.1, 0.6), Blip::new(0.7, 0.3)]).phase(0.3),
    hex_map: HexMap::new(4, (0..12).map(|i| f64::from(i) / 12.0)),
    paired: PairedBars::new([PairedBarsItem::new("mean", 3.2, 1.8), PairedBarsItem::new("over", 12.0, 4.5)]),
    bullet: Bullet::new([BulletItem::new("Revenue", 270.0).target(300.0)]),
    rose: RoseChart::new([RoseItem::new("N", 4.0), RoseItem::new("E", 9.0), RoseItem::new("S", 2.0)]),
    radial: RadialBar::new([RadialRing::new("CPU", 72.0, 100.0), RadialRing::new("RAM", 48.0, 100.0)]),
    ring_stack: RingStack::new([("CPU", 72.0), ("RAM", 48.0), ("Disk", 91.0)]),
    radar: RadarChart::new(["Speed", "Power", "Armor"]).series(RadarSeries::new("A", [3.0, 7.0, 5.0])),
    gauge: Gauge::new(0.62),
    stacked_area: StackedAreaChart::new([AreaLayer::new("a", [1.0, 3.0, 2.0]), AreaLayer::new("b", [2.0, 1.0, 4.0])]),
}

#[test]
fn hidden_series_drop_out_and_legend_toggles() {
    let area = Rect::new(0, 0, 60, 20);
    let layers = || {
        StackedAreaChart::new([
            AreaLayer::new("a", [1.0, 3.0, 2.0]),
            AreaLayer::new("b", [2.0, 1.0, 4.0]),
        ])
    };
    let mut buf = Buffer::empty(area);
    let h = HitMap::capture(layers(), area, &mut buf);
    let mut ui = Interaction::default();
    let leg = h
        .hits()
        .iter()
        .find(|x| x.kind == HitKind::Legend && x.series == 1)
        .unwrap();
    let click = ui.on_click(leg.rect.x, leg.rect.y, &h);
    assert_eq!(
        click,
        Click::Toggled {
            series: 1,
            hidden: true
        }
    );
    let mut buf = Buffer::empty(area);
    let h2 = HitMap::capture(layers().hidden(ui.hidden()), area, &mut buf);
    assert!(h2
        .hits()
        .iter()
        .all(|x| x.kind == HitKind::Legend || x.series == 0));
    assert!(h2
        .hits()
        .iter()
        .any(|x| x.kind == HitKind::Legend && x.series == 1));

    let radar = || {
        RadarChart::new(["a", "b", "c"])
            .series(RadarSeries::new("A", [3.0, 7.0, 5.0]))
            .series(RadarSeries::new("B", [4.0, 2.0, 6.0]))
    };
    let mut buf = Buffer::empty(area);
    let h = HitMap::capture(radar().hidden([0]), area, &mut buf);
    assert!(h
        .hits()
        .iter()
        .filter(|x| x.kind == HitKind::Point)
        .all(|x| x.series == 1));
}

#[test]
fn capture_does_not_leak_between_renders() {
    let area = Rect::new(0, 0, 40, 10);
    let mut buf = Buffer::empty(area);
    let a = HitMap::capture(Gauge::new(0.5), area, &mut buf);
    let b = HitMap::capture(Sparkline::new([1.0, 2.0]), area, &mut buf);
    assert_eq!(a.hits().len(), 1);
    assert!(b.hits().iter().all(|h| h.title.starts_with('#')));
    // plain render records nothing and stays cheap
    Gauge::new(0.5).render(area, &mut buf);
    assert!(HitMap::capture(ratatui::widgets::Clear, area, &mut buf).is_empty());
}

#[test]
fn funnel_and_radial_toggle_via_legend() {
    let area = Rect::new(0, 0, 70, 16);
    let funnel = || {
        Funnel::new([
            FunnelStage::new("Visits", 1000.0),
            FunnelStage::new("Signups", 400.0),
            FunnelStage::new("Paid", 100.0),
        ])
    };
    let mut buf = Buffer::empty(area);
    let h = HitMap::capture(funnel(), area, &mut buf);
    assert_eq!(
        h.hits()
            .iter()
            .filter(|x| x.kind == HitKind::Legend)
            .count(),
        3
    );
    let mut buf = Buffer::empty(area);
    let h = HitMap::capture(funnel().hidden([1]), area, &mut buf);
    assert!(h
        .hits()
        .iter()
        .all(|x| x.series != 1 || x.kind == HitKind::Legend));
    assert_eq!(
        h.hits().iter().filter(|x| x.kind == HitKind::Point).count(),
        2
    );

    let rings = || {
        RadialBar::new([
            RadialRing::new("CPU", 72.0, 100.0),
            RadialRing::new("RAM", 48.0, 100.0),
        ])
    };
    let mut buf = Buffer::empty(area);
    let h = HitMap::capture(rings().hidden([0]), area, &mut buf);
    assert!(h
        .hits()
        .iter()
        .any(|x| x.kind == HitKind::Legend && x.series == 0));
    assert!(h
        .hits()
        .iter()
        .all(|x| x.series != 0 || x.kind == HitKind::Legend));
}

#[test]
fn view_zoom_applies_to_data() {
    let mut v = View::default();
    v.on_scroll(30, Rect::new(0, 0, 60, 10), 3);
    let data: Vec<u32> = (0..200).collect();
    let part = v.slice(&data);
    assert!(part.len() < 200 && part.contains(&100));
}

#[test]
fn cyber_builders() {
    use ratatui::style::Color;
    let area = Rect::new(0, 0, 40, 10);
    let render = |w: Treemap| {
        let mut b = Buffer::empty(area);
        w.render(area, &mut b);
        b.content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect::<String>()
    };
    let items = || [TreemapItem::new("a", 5.0), TreemapItem::new("b", 3.0)];
    assert!(!render(Treemap::new(items())).contains('▟'));
    let cut = render(Treemap::new(items()).chamfered(true));
    assert!(cut.contains('▟') && cut.contains('▛'));

    // zones: 0.7 of the range is amber
    let g = Gauge::new(70.0).range(0.0, 100.0).zones();
    let mut b = Buffer::empty(area);
    g.render(area, &mut b);
    assert!(b.content().iter().any(|c| c.fg == Color::Yellow));

    // builder-style effects behave like Fx
    let mut a = Buffer::empty(area);
    let mut c = Buffer::empty(area);
    Spectrum::new([0.5, 0.9]).glow(1, 0.3).render(area, &mut a);
    Fx::new(Spectrum::new([0.5, 0.9]))
        .glow(1, 0.3)
        .render(area, &mut c);
    assert_eq!(a, c);
}
