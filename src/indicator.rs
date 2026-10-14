//! Technical indicators computed over [`Candle`] slices.
//!
//! Every function returns series aligned with the input (one entry per
//! candle, `None` while there is not enough history or data is non-finite).

use crate::candle::Candle;

/// A series aligned to the candle index.
pub type Series1 = Vec<Option<f64>>;

fn finite(v: f64) -> Option<f64> {
    v.is_finite().then_some(v)
}

/// EMA over an optional series; seeded with the SMA of the first `period`
/// consecutive values once they exist.
fn ema_opt(values: &[Option<f64>], period: usize) -> Series1 {
    let period = period.max(1);
    let k = 2.0 / (period as f64 + 1.0);
    let mut out = vec![None; values.len()];
    let mut run: Vec<f64> = vec![];
    let mut prev: Option<f64> = None;
    for (i, v) in values.iter().enumerate() {
        let Some(v) = v else {
            run.clear();
            prev = None;
            continue;
        };
        let e = match prev {
            Some(p) => v * k + p * (1.0 - k),
            None => {
                run.push(*v);
                if run.len() < period {
                    continue;
                }
                run.iter().sum::<f64>() / period as f64
            }
        };
        prev = Some(e);
        out[i] = finite(e);
    }
    out
}

/// Bollinger bands over closes: `(middle, upper, lower)` using the SMA and
/// `k` population standard deviations.
pub fn bollinger(candles: &[Candle], period: usize, k: f64) -> (Series1, Series1, Series1) {
    let period = period.max(1);
    let n = candles.len();
    let (mut mid, mut up, mut lo) = (vec![None; n], vec![None; n], vec![None; n]);
    for i in (period.saturating_sub(1))..n {
        let w = &candles[i + 1 - period..=i];
        let m = w.iter().map(|c| c.close).sum::<f64>() / period as f64;
        let var = w.iter().map(|c| (c.close - m).powi(2)).sum::<f64>() / period as f64;
        let sd = var.sqrt();
        mid[i] = finite(m);
        up[i] = finite(m + k * sd);
        lo[i] = finite(m - k * sd);
    }
    (mid, up, lo)
}

/// Relative Strength Index (Wilder smoothing), 0‥100.
pub fn rsi(candles: &[Candle], period: usize) -> Series1 {
    let period = period.max(1);
    let n = candles.len();
    let mut out = vec![None; n];
    if n <= period {
        return out;
    }
    let (mut ag, mut al) = (0.0, 0.0);
    for i in 1..n {
        let d = candles[i].close - candles[i - 1].close;
        let (g, l) = (d.max(0.0), (-d).max(0.0));
        if i <= period {
            ag += g / period as f64;
            al += l / period as f64;
        } else {
            ag = (ag * (period as f64 - 1.0) + g) / period as f64;
            al = (al * (period as f64 - 1.0) + l) / period as f64;
        }
        if i >= period {
            let v = if al == 0.0 {
                if ag == 0.0 {
                    50.0
                } else {
                    100.0
                }
            } else {
                100.0 - 100.0 / (1.0 + ag / al)
            };
            out[i] = finite(v);
        }
    }
    out
}

/// MACD: `(macd line, signal line, histogram)`.
pub fn macd(
    candles: &[Candle],
    fast: usize,
    slow: usize,
    signal: usize,
) -> (Series1, Series1, Series1) {
    let closes: Vec<Option<f64>> = candles.iter().map(|c| finite(c.close)).collect();
    let f = ema_opt(&closes, fast);
    let s = ema_opt(&closes, slow);
    let line: Series1 = f
        .iter()
        .zip(&s)
        .map(|(a, b)| a.zip(*b).map(|(a, b)| a - b))
        .collect();
    let sig = ema_opt(&line, signal);
    let hist = line
        .iter()
        .zip(&sig)
        .map(|(a, b)| a.zip(*b).map(|(a, b)| a - b))
        .collect();
    (line, sig, hist)
}

/// Average True Range (Wilder smoothing).
pub fn atr(candles: &[Candle], period: usize) -> Series1 {
    let period = period.max(1);
    let n = candles.len();
    let mut out = vec![None; n];
    if n < period {
        return out;
    }
    let tr = |i: usize| {
        let c = &candles[i];
        if i == 0 {
            c.high - c.low
        } else {
            let pc = candles[i - 1].close;
            (c.high - c.low)
                .max((c.high - pc).abs())
                .max((c.low - pc).abs())
        }
    };
    let mut a = (0..period).map(tr).sum::<f64>() / period as f64;
    out[period - 1] = finite(a);
    for (i, o) in out.iter_mut().enumerate().skip(period) {
        a = (a * (period as f64 - 1.0) + tr(i)) / period as f64;
        *o = finite(a);
    }
    out
}

/// Cumulative volume-weighted average price (typical price × volume).
pub fn vwap(candles: &[Candle]) -> Series1 {
    let (mut pv, mut v) = (0.0, 0.0);
    candles
        .iter()
        .map(|c| {
            let tp = (c.high + c.low + c.close) / 3.0;
            if tp.is_finite() && c.volume.is_finite() {
                pv += tp * c.volume;
                v += c.volume;
            }
            if v > 0.0 {
                finite(pv / v)
            } else {
                None
            }
        })
        .collect()
}

/// Stochastic oscillator: `(%K, %D)` where %D is the SMA of %K.
pub fn stochastic(candles: &[Candle], k_period: usize, d_period: usize) -> (Series1, Series1) {
    let kp = k_period.max(1);
    let dp = d_period.max(1);
    let n = candles.len();
    let mut k = vec![None; n];
    for i in (kp - 1)..n {
        let w = &candles[i + 1 - kp..=i];
        let hh = w.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
        let ll = w.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
        k[i] = if hh > ll {
            finite((candles[i].close - ll) / (hh - ll) * 100.0)
        } else {
            Some(50.0)
        };
    }
    let mut d = vec![None; n];
    for i in 0..n {
        if i + 1 < dp {
            continue;
        }
        let w = &k[i + 1 - dp..=i];
        if w.iter().all(|v| v.is_some()) {
            d[i] = Some(w.iter().flatten().sum::<f64>() / dp as f64);
        }
    }
    (k, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closes(v: &[f64]) -> Vec<Candle> {
        v.iter()
            .map(|&p| Candle::new(p, p, p, p).volume(10.0))
            .collect()
    }

    #[test]
    fn bollinger_flat_has_zero_width() {
        let (m, u, l) = bollinger(&closes(&[5.0; 6]), 3, 2.0);
        assert_eq!(m[1], None);
        assert_eq!(m[2], Some(5.0));
        assert_eq!(u[5], l[5]);
    }

    #[test]
    fn bollinger_values() {
        let (m, u, l) = bollinger(&closes(&[1.0, 3.0]), 2, 1.0);
        assert_eq!(m[1], Some(2.0));
        assert_eq!(u[1], Some(3.0));
        assert_eq!(l[1], Some(1.0));
    }

    #[test]
    fn rsi_extremes() {
        let up = rsi(&closes(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3);
        assert_eq!(up[2], None);
        assert_eq!(up[3], Some(100.0));
        let dn = rsi(&closes(&[5.0, 4.0, 3.0, 2.0, 1.0]), 3);
        assert_eq!(dn[4], Some(0.0));
        assert_eq!(rsi(&closes(&[1.0]), 14), vec![None]);
    }

    #[test]
    fn macd_aligned_and_hist() {
        let d: Vec<f64> = (0..40)
            .map(|i| 100.0 + (i as f64 * 0.4).sin() * 5.0)
            .collect();
        let c = closes(&d);
        let (m, s, h) = macd(&c, 12, 26, 9);
        assert_eq!((m.len(), s.len(), h.len()), (40, 40, 40));
        assert!(m[24].is_none() && m[25].is_some());
        assert!(s[32].is_none() && s[33].is_some());
        let i = 35;
        assert!((h[i].unwrap() - (m[i].unwrap() - s[i].unwrap())).abs() < 1e-12);
    }

    #[test]
    fn atr_constant_range() {
        let c: Vec<Candle> = (0..6).map(|_| Candle::new(10.0, 12.0, 8.0, 10.0)).collect();
        let a = atr(&c, 3);
        assert_eq!(a[1], None);
        assert_eq!(a[2], Some(4.0));
        assert_eq!(a[5], Some(4.0));
    }

    #[test]
    fn vwap_weights() {
        let c = vec![
            Candle::new(10.0, 10.0, 10.0, 10.0).volume(1.0),
            Candle::new(20.0, 20.0, 20.0, 20.0).volume(3.0),
        ];
        let v = vwap(&c);
        assert_eq!(v[0], Some(10.0));
        assert_eq!(v[1], Some(17.5));
        assert_eq!(
            vwap(
                &closes(&[1.0])
                    .into_iter()
                    .map(|c| c.volume(0.0))
                    .collect::<Vec<_>>()
            )[0],
            None
        );
    }

    #[test]
    fn stochastic_range() {
        let c = vec![
            Candle::new(1.0, 2.0, 0.0, 1.0),
            Candle::new(1.0, 4.0, 1.0, 4.0),
            Candle::new(4.0, 4.0, 2.0, 2.0),
        ];
        let (k, d) = stochastic(&c, 2, 2);
        assert_eq!(k[0], None);
        assert_eq!(k[1], Some(100.0));
        assert!((k[2].unwrap() - 100.0 / 3.0).abs() < 1e-9);
        assert!((d[2].unwrap() - (100.0 + 100.0 / 3.0) / 2.0).abs() < 1e-9);
    }

    #[test]
    fn empty_and_nan_safe() {
        assert!(rsi(&[], 5).is_empty());
        assert!(atr(&[], 5).is_empty());
        assert!(vwap(&[]).is_empty());
        let (a, _, _) = macd(&[], 3, 5, 2);
        assert!(a.is_empty());
        let c = closes(&[1.0, f64::NAN, 3.0, 4.0]);
        let _ = (
            bollinger(&c, 2, 2.0),
            rsi(&c, 2),
            macd(&c, 2, 3, 2),
            stochastic(&c, 2, 2),
        );
    }
}
