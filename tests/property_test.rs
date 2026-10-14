//! Property-style tests driven by a tiny deterministic xorshift RNG.

use tui_chart::util::{format_compact, format_tick, nice_bounds, nice_step, nice_ticks};
use tui_chart::*;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// Uniform in `[0, 1)`.
    fn f(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.f()
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const N: usize = 500;

fn random_candles(r: &mut Rng, n: usize) -> Vec<Candle> {
    let mut p = r.range(1.0, 1000.0);
    (0..n)
        .map(|_| {
            let o = p;
            let c = (o + r.range(-5.0, 5.0)).max(0.01);
            let h = o.max(c) + r.range(0.0, 3.0);
            let l = (o.min(c) - r.range(0.0, 3.0)).max(0.001);
            p = c;
            Candle::new(o, h, l, c).volume(r.range(1.0, 1e6))
        })
        .collect()
}

#[test]
fn nice_ticks_are_monotone_inside_range() {
    let mut r = Rng(0x9E3779B97F4A7C15);
    for _ in 0..N {
        let a = r.range(-1e6, 1e6);
        let b = a + r.range(1e-6, 1e6);
        let count = 2 + r.below(12);
        let t = nice_ticks(a, b, count);
        if t.is_empty() {
            continue; // no nice multiple inside a very narrow window
        }
        for w in t.windows(2) {
            assert!(w[1] > w[0], "not strictly increasing: {t:?}");
        }
        let eps = (b - a) * 1e-6;
        assert!(
            t[0] >= a - eps && *t.last().unwrap() <= b + eps,
            "{a} {b} {t:?}"
        );
        // evenly spaced
        let s = t[1.min(t.len() - 1)] - t[0];
        for w in t.windows(2) {
            assert!(((w[1] - w[0]) - s).abs() <= s.abs() * 1e-6 + 1e-9);
        }
    }
}

#[test]
fn nice_bounds_contain_input_and_step_is_nice() {
    let mut r = Rng(42);
    for _ in 0..N {
        let a = r.range(-1e4, 1e4);
        let b = a + r.range(1e-3, 1e4);
        let (lo, hi) = nice_bounds(a, b, 5);
        assert!(lo <= a + 1e-9 && hi >= b - 1e-9 && lo < hi);
        let s = nice_step(r.range(1e-6, 1e6));
        let m = s / 10f64.powf(s.log10().floor());
        assert!(
            [1.0, 2.0, 2.5, 5.0, 10.0]
                .iter()
                .any(|n| (m - n).abs() < 1e-9),
            "{s}"
        );
    }
}

#[test]
fn degenerate_inputs_never_panic() {
    let specials = [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        0.0,
        -0.0,
        1e300,
        -1e300,
        1e-300,
    ];
    for &a in &specials {
        for &b in &specials {
            let _ = nice_ticks(a, b, 5);
            let _ = nice_bounds(a, b, 5);
            let _ = format_tick(a, b);
            let _ = time_ticks(a, b, 5);
        }
        let _ = format_compact(a);
        let _ = nice_step(a);
        let _ = time_hms(a);
        let _ = datetime(a);
        let _ = date_ymd(a);
    }
}

#[test]
fn civil_date_round_trips() {
    let mut r = Rng(7);
    for _ in 0..N {
        let days = (r.next() % 400_000) as i64 - 200_000;
        let (y, m, d) = civil_from_days(days);
        assert!((1..=12).contains(&m) && (1..=31).contains(&d));
        assert_eq!(days_from_civil(y, m, d), days);
    }
    assert_eq!(civil_from_days(0), (1970, 1, 1));
}

#[test]
fn time_ticks_monotone_and_within_range() {
    let mut r = Rng(1234);
    for _ in 0..N {
        let a = r.range(0.0, 2e9);
        let span = r.range(1.0, 5e8);
        let b = a + span;
        let t = time_ticks(a, b, 2 + r.below(10));
        for w in t.windows(2) {
            assert!(w[1] > w[0], "{t:?}");
        }
        for &x in &t {
            assert!(x >= a - 1e-6 && x <= b + 1e-6, "{x} outside [{a},{b}]");
        }
        assert!(time_nice_step(span, 6) > 0.0);
    }
}

#[test]
fn ring_buffer_never_exceeds_capacity_and_keeps_order() {
    let mut r = Rng(99);
    for _ in 0..50 {
        let cap = 1 + r.below(20);
        let mut ring = RingBuffer::new(cap);
        let mut all = Vec::new();
        for i in 0..r.below(100) {
            let evicted = ring.push(i);
            all.push(i);
            assert!(ring.len() <= ring.capacity());
            assert_eq!(ring.capacity(), cap);
            assert_eq!(evicted.is_some(), all.len() > cap);
            let tail: Vec<_> = all.iter().rev().take(cap).rev().copied().collect();
            assert_eq!(ring.iter().copied().collect::<Vec<_>>(), tail);
            assert_eq!(ring.last(), all.last());
        }
        ring.clear();
        assert!(ring.is_empty());
    }
    assert_eq!(
        RingBuffer::<u8>::new(0).capacity(),
        1,
        "capacity clamps to 1"
    );
}

#[test]
fn stream_x_values_increase_and_ranges_bound_points() {
    let mut r = Rng(5);
    let mut s = Stream::new(16).x_start(100.0).x_step(0.5);
    for i in 0..200 {
        s.push(r.range(-50.0, 50.0));
        assert!(s.len() <= 16);
        let pts: Vec<_> = s.points().collect();
        for w in pts.windows(2) {
            assert!(w[1].0 > w[0].0);
        }
        let (ylo, yhi) = s.y_range().unwrap();
        assert!(pts.iter().all(|p| p.1 >= ylo && p.1 <= yhi));
        assert_eq!(pts.last().unwrap().0, 100.0 + 0.5 * i as f64);
    }
}

#[test]
fn rsi_in_0_100_and_aligned() {
    let mut r = Rng(31337);
    for _ in 0..50 {
        let n = r.below(80);
        let cs = random_candles(&mut r, n);
        let period = 1 + r.below(20);
        let v = rsi(&cs, period);
        assert_eq!(v.len(), cs.len());
        for x in v.iter().flatten() {
            assert!((0.0..=100.0).contains(x), "rsi {x}");
        }
    }
}

#[test]
fn bollinger_ordering() {
    let mut r = Rng(2024);
    for _ in 0..50 {
        let n = 5 + r.below(80);
        let cs = random_candles(&mut r, n);
        let (mid, up, lo) = bollinger(&cs, 1 + r.below(15), r.range(0.0, 4.0));
        assert!(up.len() == cs.len() && mid.len() == cs.len() && lo.len() == cs.len());
        for i in 0..cs.len() {
            if let (Some(u), Some(m), Some(l)) = (up[i], mid[i], lo[i]) {
                assert!(u >= m - 1e-9 && m >= l - 1e-9, "{u} {m} {l}");
            } else {
                assert!(up[i].is_none() && mid[i].is_none() && lo[i].is_none());
            }
        }
    }
}

#[test]
fn stochastic_atr_vwap_sma_ema_invariants() {
    let mut r = Rng(77);
    for _ in 0..50 {
        let n = 5 + r.below(80);
        let cs = random_candles(&mut r, n);
        let (k, d) = stochastic(&cs, 1 + r.below(14), 1 + r.below(5));
        for x in k.iter().chain(d.iter()).flatten() {
            assert!((-1e-9..=100.0 + 1e-9).contains(x), "stoch {x}");
        }
        for x in atr(&cs, 1 + r.below(14)).iter().flatten() {
            assert!(*x >= 0.0);
        }
        let lo = cs.iter().map(|c| c.low).fold(f64::MAX, f64::min);
        let hi = cs.iter().map(|c| c.high).fold(f64::MIN, f64::max);
        for x in vwap(&cs).iter().flatten() {
            assert!(
                *x >= lo - 1e-6 && *x <= hi + 1e-6,
                "vwap {x} not in [{lo},{hi}]"
            );
        }
        let p = 1 + r.below(10);
        let (s, e) = (sma(&cs, p), ema(&cs, p));
        assert!(s.len() == cs.len() && e.len() == cs.len());
        let cmin = cs.iter().map(|c| c.close).fold(f64::MAX, f64::min);
        let cmax = cs.iter().map(|c| c.close).fold(f64::MIN, f64::max);
        for x in s.iter().chain(e.iter()).flatten() {
            assert!(*x >= cmin - 1e-6 && *x <= cmax + 1e-6);
        }
        let m = macd(&cs, 3, 8, 4);
        assert_eq!(m.0.len(), cs.len());
    }
}

#[test]
fn heikin_ashi_preserves_length_and_bounds() {
    let mut r = Rng(11);
    let cs = random_candles(&mut r, 60);
    let ha = heikin_ashi(&cs);
    assert_eq!(ha.len(), cs.len());
    for c in &ha {
        assert!(c.high >= c.open.max(c.close) - 1e-9);
        assert!(c.low <= c.open.min(c.close) + 1e-9);
    }
}

#[test]
fn renko_bricks_are_contiguous() {
    let mut r = Rng(8);
    for _ in 0..50 {
        let mut p = 100.0;
        let closes: Vec<f64> = (0..60)
            .map(|_| {
                p += r.range(-4.0, 4.0);
                p
            })
            .collect();
        let size = r.range(0.5, 3.0);
        let b = renko_bricks(&closes, size);
        for w in b.windows(2) {
            assert!(
                (w[1].open - w[0].close).abs() < 1e-6 || (w[1].open - w[0].open).abs() < 1e-6,
                "gap between bricks"
            );
        }
        for br in &b {
            assert!(((br.close - br.open).abs() - size).abs() < 1e-6);
        }
    }
}

#[test]
fn treemap_layout_partitions_area() {
    let mut r = Rng(3);
    for _ in 0..100 {
        let n = 1 + r.below(12);
        let w: Vec<f64> = (0..n).map(|_| r.range(0.1, 100.0)).collect();
        let area =
            ratatui::layout::Rect::new(0, 0, 20 + r.below(100) as u16, 5 + r.below(40) as u16);
        let rects = Treemap::layout(&w, area);
        assert_eq!(rects.len(), n);
        for rc in &rects {
            assert!(rc.x >= area.x && rc.right() <= area.right());
            assert!(rc.y >= area.y && rc.bottom() <= area.bottom());
        }
        let total: u32 = rects.iter().map(|r| r.width as u32 * r.height as u32).sum();
        assert!(total <= area.width as u32 * area.height as u32);
    }
}

#[test]
fn random_candle_charts_never_panic() {
    use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
    let mut r = Rng(555);
    for _ in 0..30 {
        let n = r.below(120);
        let cs = random_candles(&mut r, n);
        let (w, h) = (1 + r.below(120) as u16, 1 + r.below(40) as u16);
        let a = Rect::new(0, 0, w, h);
        let mut b = Buffer::empty(a);
        CandlestickChart::new(cs)
            .show_volume(r.below(2) == 0)
            .render(a, &mut b);
    }
}
