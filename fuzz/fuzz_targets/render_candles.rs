#![no_main]
mod common;
use arbitrary::Arbitrary;
use common::*;
use libfuzzer_sys::fuzz_target;
use tui_chart::*;

#[derive(Arbitrary, Debug)]
struct Input {
    candles: Vec<CandleIn>,
    style: u8,
    hollow: bool,
    theme: u8,
    candle_width: u8,
    gap: u8,
    volume: bool,
    last_price: bool,
    scroll: u16,
    overlay: Vec<Option<f64>>,
    band: (Vec<Option<f64>>, Vec<Option<f64>>),
    boll: (u8, f64),
    panes: Vec<(u8, u8, u8, u8)>,
    markers: Vec<(u16, f64, bool)>,
    hlines: Vec<f64>,
    support: Option<f64>,
    profile: bool,
    x: AxisIn,
    y: AxisIn,
    color: Col,
    size: Size,
}

fuzz_target!(|i: Input| {
    let cs = candles(&i.candles);
    let col = i.color.get();
    let mut c = CandlestickChart::new(cs.clone())
        .style(match i.style % 4 {
            0 => CandleStyle::Candle,
            1 => CandleStyle::Ohlc,
            2 => CandleStyle::HeikinAshi,
            _ => CandleStyle::Braille,
        })
        .hollow(i.hollow)
        .theme(theme(i.theme))
        .candle_width(i.candle_width as u16)
        .gap(i.gap as u16)
        .show_volume(i.volume)
        .last_price_line(i.last_price)
        .scroll(i.scroll as usize)
        .volume_profile(i.profile)
        .overlay(Overlay::new("o", cap(&i.overlay), col))
        .overlay(Overlay::new("sma", sma(&cs, i.boll.0 as usize), col))
        .overlay(Overlay::new("ema", ema(&cs, i.boll.0 as usize), col))
        .band(BandOverlay::new("b", cap(&i.band.0), cap(&i.band.1), col))
        .band(BandOverlay::bollinger(&cs, i.boll.0 as usize, i.boll.1, col))
        .x_axis(i.x.get())
        .y_axis(i.y.get());
    for (k, a, b, d) in i.panes.iter().take(4) {
        c = c.pane(match k % 3 {
            0 => IndicatorPane::Rsi { period: *a as usize },
            1 => IndicatorPane::Macd {
                fast: *a as usize,
                slow: *b as usize,
                signal: *d as usize,
            },
            _ => IndicatorPane::Custom {
                name: "c".into(),
                values: cs.iter().map(|x| Some(x.close)).collect(),
                color: col,
            },
        });
    }
    for (idx, p, buy) in i.markers.iter().take(32) {
        let side = if *buy { TradeSide::Buy } else { TradeSide::Sell };
        c = c.marker(TradeMarker::new(*idx as usize, *p, side).label("m"));
    }
    for p in i.hlines.iter().take(16) {
        c = c.hline(HLine::new(*p, col));
    }
    if let Some(p) = i.support {
        c = c.support(p).resistance(p);
    }
    let _ = heikin_ashi(&cs);
    render(c, i.size);
});
