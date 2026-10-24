#![no_main]
mod common;
use arbitrary::Arbitrary;
use common::*;
use libfuzzer_sys::fuzz_target;
use ratatui::style::Color;
use tui_chart::*;

#[derive(Arbitrary, Debug)]
enum Input {
    Instrument {
        values: Vec<f64>,
        peaks: Vec<f64>,
        cols: u8,
        phase: f64,
        kind: u8,
        size: Size,
    },
    Radial {
        rings: Vec<(String, f64, f64)>,
        full: bool,
        legend: bool,
        stack: bool,
        size: Size,
    },
    Bars {
        groups: Vec<(String, Vec<f64>)>,
        horizontal: bool,
        stacked: bool,
        bar_width: u8,
        gaps: (u8, u8),
        values: bool,
        inside: bool,
        max: Option<f64>,
        marker: MarkerIn,
        size: Size,
    },
    Histogram {
        data: Vec<f64>,
        bins: u8,
        range: Option<(f64, f64)>,
        kde: bool,
        cumulative: bool,
        normalized: bool,
        horizontal: bool,
        size: Size,
    },
    Heatmap {
        rows: Vec<Vec<f64>>,
        cell: (u8, u8),
        bounds: Option<(f64, f64)>,
        grad: u8,
        custom: Vec<(u8, u8, u8)>,
        values: bool,
        legend: bool,
        size: Size,
    },
    Treemap {
        items: Vec<(String, f64, Vec<(String, f64)>)>,
        border: bool,
        values: bool,
        size: Size,
    },
    BoxPlot {
        stats: Vec<(f64, f64, f64, f64, f64, Vec<f64>)>,
        samples: Vec<f64>,
        k: f64,
        horizontal: bool,
        range: Option<(f64, f64)>,
        size: Size,
    },
    Waterfall {
        steps: Vec<(f64, bool)>,
        initial: f64,
        size: Size,
    },
    Gantt {
        tasks: Vec<(String, f64, f64, f64)>,
        today: Option<f64>,
        range: Option<(f64, f64)>,
        size: Size,
    },
    Depth {
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        probe: f64,
        size: Size,
    },
    Renko {
        closes: Vec<f64>,
        brick: f64,
        pnf: bool,
        width: u8,
        size: Size,
    },
    Radar {
        axes: Vec<String>,
        series: Vec<Vec<f64>>,
        max: Option<f64>,
        rings: u8,
        fill: bool,
        size: Size,
    },
    Gauge {
        value: f64,
        range: (f64, f64),
        bands: Vec<(f64, Col)>,
        target: Option<f64>,
        orient: u8,
        style: u8,
        size: Size,
    },
    Sparkline {
        data: Vec<f64>,
        min: Option<f64>,
        max: Option<f64>,
        baseline: Option<f64>,
        style: u8,
        marker: MarkerIn,
        size: Size,
    },
}

fn gradient(g: u8, custom: &[(u8, u8, u8)]) -> Gradient {
    match g % 6 {
        0 => Gradient::Viridis,
        1 => Gradient::Heat,
        2 => Gradient::Cool,
        3 => Gradient::RedBlue,
        4 => Gradient::Grayscale,
        _ => Gradient::Custom(cap(custom)),
    }
}

fn orient(h: bool) -> BarOrientation {
    if h {
        BarOrientation::Horizontal
    } else {
        BarOrientation::Vertical
    }
}

fuzz_target!(|i: Input| match i {
    Input::Bars { groups, horizontal, stacked, bar_width, gaps, values, inside, max, marker, size } => {
        let g = groups
            .iter()
            .take(200)
            .map(|(l, v)| BarGroup::new(l.clone(), v.iter().copied().take(16)))
            .collect();
        let mut c = BarChart::new(g)
            .orientation(orient(horizontal))
            .mode(if stacked { BarMode::Stacked } else { BarMode::Grouped })
            .bar_width(bar_width as u16)
            .bar_gap(gaps.0 as u16)
            .group_gap(gaps.1 as u16)
            .show_values(values)
            .label_position(if inside { LabelPosition::Inside } else { LabelPosition::Outside })
            .marker(marker.get());
        if let Some(m) = max {
            c = c.max(m);
        }
        let _ = c.bounds();
        render(c, size);
    }
    Input::Histogram { data, bins, range, kde, cumulative, normalized, horizontal, size } => {
        let mut h = Histogram::new(data.into_iter().take(MAX_POINTS))
            .bins(bins as usize)
            .kde(kde)
            .cumulative(cumulative)
            .normalized(normalized)
            .orientation(orient(horizontal));
        if let Some((a, b)) = range {
            h = h.range(a, b);
        }
        let _ = (h.compute_bins(), h.heights(), h.kde_curve(32, None));
        render(h, size);
    }
    Input::Heatmap { rows, cell, bounds, grad, custom, values, legend, size } => {
        let data: Vec<Vec<f64>> = rows
            .into_iter()
            .take(100)
            .map(|r| r.into_iter().take(100).collect())
            .collect();
        let mut h = Heatmap::new(data)
            .gradient(gradient(grad, &custom))
            .cell_width(cell.0 as u16)
            .cell_height(cell.1 as u16)
            .show_values(values)
            .legend(legend);
        if let Some((a, b)) = bounds {
            h = h.bounds(a, b);
        }
        let _ = h.scale();
        render(h, size);
    }
    Input::Treemap { items, border, values, size } => {
        let it: Vec<TreemapItem> = items
            .into_iter()
            .take(100)
            .map(|(l, v, ch)| {
                TreemapItem::new(l, v)
                    .children(ch.into_iter().take(20).map(|(l, v)| TreemapItem::new(l, v)))
            })
            .collect();
        let w: Vec<f64> = it.iter().map(|x| x.weight()).collect();
        let _ = Treemap::layout(&w, size.area());
        render(Treemap::new(it).border(border).show_values(values), size);
    }
    Input::BoxPlot { stats, samples, k, horizontal, range, size } => {
        let mut b = BoxPlot::new().whisker_iqr(k).orientation(if horizontal {
            BoxOrientation::Horizontal
        } else {
            BoxOrientation::Vertical
        });
        for (n, (a, q1, m, q3, z, o)) in stats.into_iter().take(50).enumerate() {
            b = b.stats(
                format!("g{n}"),
                BoxStats::new(a, q1, m, q3, z).outliers(o.into_iter().take(50)),
            );
        }
        let s = cap(&samples);
        if let Some(st) = BoxStats::from_samples(&s, k) {
            b = b.stats("s", st);
        }
        if let Some((a, z)) = range {
            b = b.range(a, z);
        }
        render(b, size);
    }
    Input::Waterfall { steps, initial, size } => {
        let mut w = Waterfall::new().initial(initial);
        for (n, (v, total)) in steps.into_iter().take(200).enumerate() {
            w = if total { w.total(format!("t{n}")) } else { w.delta(format!("d{n}"), v) };
        }
        let _ = w.levels();
        render(w, size);
    }
    Input::Gantt { tasks, today, range, size } => {
        let t = tasks
            .into_iter()
            .take(200)
            .map(|(l, s, e, p)| Task::new(l, s, e).progress(p));
        let mut g = Gantt::new(t);
        if let Some(x) = today {
            g = g.today(x);
        }
        if let Some((a, b)) = range {
            g = g.range(a, b);
        }
        render(g, size);
    }
    Input::Depth { bids, asks, probe, size } => {
        let d = DepthChart::new(
            bids.into_iter().take(MAX_POINTS),
            asks.into_iter().take(MAX_POINTS),
        );
        let _ = (d.mid(), d.spread(), d.bid_depth_at(probe), d.ask_depth_at(probe));
        render(d, size);
    }
    Input::Renko { closes, brick, pnf, width, size } => {
        let c = cap(&closes);
        let _ = renko_bricks(&c, brick);
        let r = RenkoChart::new(&c, brick)
            .style(if pnf { RenkoStyle::PointAndFigure } else { RenkoStyle::Bricks })
            .brick_width(width as u16);
        let _ = r.columns();
        render(r, size);
    }
    Input::Radar { axes, series, max, rings, fill, size } => {
        let mut r = RadarChart::new(axes.into_iter().take(32))
            .rings(rings as usize)
            .fill(fill);
        for (n, s) in series.into_iter().take(8).enumerate() {
            r = r.series(RadarSeries::new(format!("s{n}"), s.into_iter().take(64)));
        }
        if let Some(m) = max {
            r = r.max(m);
        }
        render(r, size);
    }
    Input::Radial { rings, full, legend, stack, size } => {
        let rings: Vec<_> = rings.into_iter().take(32).collect();
        if stack {
            render(
                RingStack::new(rings.into_iter().map(|(l, v, _)| (l, v))).legend(legend),
                size,
            );
        } else {
            let sweep = if full { RadialSweep::Full } else { RadialSweep::Arc };
            let rings = rings.into_iter().map(|(l, v, m)| RadialRing::new(l, v, m));
            render(RadialBar::new(rings).sweep(sweep).legend(legend), size);
        }
    }
    Input::Instrument { values, peaks, cols, phase, kind, size } => {
        let values: Vec<f64> = values.into_iter().take(256).collect();
        match kind % 5 {
            0 => render(Oscilloscope::new(values).phase(phase), size),
            1 => render(Spectrum::new(values).peaks(peaks), size),
            2 => render(DotMatrix::new(values).rows(usize::from(cols)), size),
            3 => {
                let b = values.chunks(2).map(|c| Blip::new(c[0], *c.get(1).unwrap_or(&0.5)));
                render(RadarScope::new(b).phase(phase), size);
            }
            _ => render(HexMap::new(usize::from(cols), values).phase(phase), size),
        }
    }
    Input::Gauge { value, range, bands, target, orient, style, size } => {
        let mut g = Gauge::new(value)
            .range(range.0, range.1)
            .bands(bands.into_iter().take(16).map(|(v, c)| (v, c.get())))
            .orientation(if orient % 2 == 0 {
                GaugeOrientation::Horizontal
            } else {
                GaugeOrientation::Vertical
            })
            .style(GaugeStyle::ALL[usize::from(style) % GaugeStyle::ALL.len()])
            .color(Color::Green);
        if let Some(t) = target {
            g = g.target(t);
        }
        render(g, size);
    }
    Input::Sparkline { data, min, max, baseline, style, marker, size } => {
        let mut s = Sparkline::new(data.into_iter().take(MAX_POINTS))
            .style(if style % 2 == 0 { SparklineStyle::Bar } else { SparklineStyle::Line })
            .marker(marker.get());
        if let Some(v) = min {
            s = s.min(v);
        }
        if let Some(v) = max {
            s = s.max(v);
        }
        if let Some(v) = baseline {
            s = s.baseline(v);
        }
        render(s, size);
    }
});
