use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use tui_chart::{
    atr, bollinger, macd, rsi, stochastic, vwap, BandOverlay, Candle, CandlestickChart, HLine,
    IndicatorPane, TradeMarker, TradeSide,
};

fn data(n: usize) -> Vec<Candle> {
    (0..n)
        .map(|i| {
            let b = 100.0 + (i as f64 * 0.5).sin() * 8.0;
            Candle::new(b, b + 3.0, b - 3.0, b + 1.0).volume(100.0 + i as f64)
        })
        .collect()
}

fn text(buf: &Buffer, a: Rect) -> String {
    (a.y..a.bottom())
        .flat_map(|y| (a.x..a.right()).map(move |x| (x, y)))
        .map(|p| buf[p].symbol().to_string())
        .collect()
}

#[test]
fn indicators_aligned() {
    let d = data(50);
    assert_eq!(bollinger(&d, 20, 2.0).0.len(), 50);
    assert_eq!(rsi(&d, 14).len(), 50);
    assert_eq!(macd(&d, 12, 26, 9).2.len(), 50);
    assert_eq!(atr(&d, 14).len(), 50);
    assert_eq!(vwap(&d).len(), 50);
    assert_eq!(stochastic(&d, 14, 3).1.len(), 50);
    assert!(rsi(&d, 14)
        .iter()
        .flatten()
        .all(|v| (0.0..=100.0).contains(v)));
}

#[test]
fn full_chart_renders() {
    let d = data(80);
    let area = Rect::new(0, 0, 90, 36);
    let mut buf = Buffer::empty(area);
    let chart = CandlestickChart::new(d.clone())
        .band(BandOverlay::bollinger(&d, 20, 2.0, Color::Blue))
        .pane(IndicatorPane::Rsi { period: 14 })
        .pane(IndicatorPane::Macd {
            fast: 12,
            slow: 26,
            signal: 9,
        })
        .marker(TradeMarker::new(70, 100.0, TradeSide::Buy))
        .hline(HLine::new(100.0, Color::Yellow).label("mid"))
        .volume_profile(true);
    (&chart).render(area, &mut buf);
    let t = text(&buf, area);
    assert!(t.contains('▲') && t.contains("RSI 14") && t.contains("mid"));
}

#[test]
fn degenerate_inputs_do_not_panic() {
    for (w, h) in [(0, 0), (5, 5), (20, 8)] {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        CandlestickChart::new(vec![])
            .pane(IndicatorPane::Rsi { period: 0 })
            .volume_profile(true)
            .render(area, &mut buf);
    }
}
