#![no_main]
mod common;
use common::*;
use libfuzzer_sys::fuzz_target;
use tui_chart::*;

fuzz_target!(|d: (Vec<CandleIn>, u8, u8, u8, f64)| {
    let (v, p, q, r, k) = d;
    let c = candles(&v);
    let (p, q, r) = (p as usize, q as usize, r as usize);
    let n = c.len();
    let ok = |s: &[Option<f64>]| assert_eq!(s.len(), n);
    ok(&sma(&c, p));
    ok(&ema(&c, p));
    ok(&rsi(&c, p));
    ok(&atr(&c, p));
    ok(&vwap(&c));
    let (a, b, h) = macd(&c, p, q, r);
    ok(&a);
    ok(&b);
    ok(&h);
    let (a, b, h) = bollinger(&c, p, k);
    ok(&a);
    ok(&b);
    ok(&h);
    let (a, b) = stochastic(&c, p, q);
    ok(&a);
    ok(&b);
    assert_eq!(heikin_ashi(&c).len(), n);
});
