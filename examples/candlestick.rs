//! Live trading chart demo.
//! Keys: s style (incl. braille) · v volume · a SMA · e EMA · ←/→ scroll · space pause · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::Color,
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{ema, sma, Axis, Candle, CandleStyle, CandlestickChart, Overlay, Theme};

struct State {
    theme: Theme,
    candles: Vec<Candle>,
    rng: Rng,
    style: CandleStyle,
    volume: bool,
    sma: bool,
    ema: bool,
    scroll: usize,
    paused: bool,
    n: usize,
}

fn next_candle(rng: &mut Rng, prev_close: f64, n: usize) -> Candle {
    let open = prev_close;
    let close = open + rng.gauss() * 1.2;
    let high = open.max(close) + rng.f64() * 1.5;
    let low = open.min(close) - rng.f64() * 1.5;
    let vol = 500.0 + rng.f64() * 2500.0 + (close - open).abs() * 400.0;
    Candle::new(open, high, low, close)
        .volume(vol)
        .label(format!("{:02}:{:02}", 9 + n / 60, n % 60))
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(7);
    let mut candles = vec![];
    let mut price = 100.0;
    for n in 0..200 {
        let c = next_candle(&mut rng, price, n);
        price = c.close;
        candles.push(c);
    }
    let state = State {
        theme: Theme::default(),
        candles,
        rng,
        style: CandleStyle::Candle,
        volume: true,
        sma: true,
        ema: false,
        scroll: 0,
        paused: false,
        n: 200,
    };
    run(
        state,
        Duration::from_millis(400),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(8), Constraint::Length(2)]).areas(f.area());
            let last = s.candles.last().map(|c| c.close).unwrap_or(0.0);
            let mut chart = CandlestickChart::new(s.candles.clone())
                .style(s.style)
                .show_volume(s.volume)
                .scroll(s.scroll)
                .y_axis(Axis::new().grid(true).title("price"))
                .x_axis(Axis::new().ticks(6))
                .theme(s.theme.clone())
                .block(Block::bordered().title(format!(" ACME · {:?} · last {last:.2} ", s.style)));
            if s.sma {
                chart = chart.overlay(Overlay::new("SMA20", sma(&s.candles, 20), Color::Yellow));
            }
            if s.ema {
                chart = chart.overlay(Overlay::new("EMA50", ema(&s.candles, 50), Color::Magenta));
            }
            f.render_widget(&chart, main);
            help(
                f,
                bar,
                " s style · v volume · a SMA · e EMA · ←/→ scroll · space pause · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('s') => {
                    s.style = match s.style {
                        CandleStyle::Candle => CandleStyle::Ohlc,
                        CandleStyle::Ohlc => CandleStyle::HeikinAshi,
                        CandleStyle::HeikinAshi => CandleStyle::Braille,
                        CandleStyle::Braille => CandleStyle::Candle,
                    }
                }
                KeyCode::Char('v') => s.volume = !s.volume,
                KeyCode::Char('a') => s.sma = !s.sma,
                KeyCode::Char('e') => s.ema = !s.ema,
                KeyCode::Left => s.scroll = (s.scroll + 5).min(s.candles.len().saturating_sub(1)),
                KeyCode::Right => s.scroll = s.scroll.saturating_sub(5),
                KeyCode::Char(' ') => s.paused = !s.paused,
                _ => {}
            }
            false
        },
        |s| {
            if s.paused {
                return;
            }
            let prev = s.candles.last().map(|c| c.close).unwrap_or(100.0);
            let c = next_candle(&mut s.rng, prev, s.n);
            s.n += 1;
            s.candles.push(c);
            if s.candles.len() > 1000 {
                s.candles.remove(0);
            }
        },
    )
}
