//! Numeric helpers: "nice" tick generation and number formatting.

/// Round a raw step up to a "nice" value (1, 2, 5 × 10ⁿ).
pub fn nice_step(raw: f64) -> f64 {
    if !raw.is_finite() || raw <= 0.0 {
        return 1.0;
    }
    let exp = raw.log10().floor();
    let base = 10f64.powf(exp);
    let f = raw / base;
    let nf = if f <= 1.0 {
        1.0
    } else if f <= 2.0 {
        2.0
    } else if f <= 2.5 {
        2.5
    } else if f <= 5.0 {
        5.0
    } else {
        10.0
    };
    nf * base
}

/// Generate roughly `count` nicely spaced tick values inside `[min, max]`.
pub fn nice_ticks(min: f64, max: f64, count: usize) -> Vec<f64> {
    if !min.is_finite() || !max.is_finite() || max <= min {
        return if min.is_finite() { vec![min] } else { vec![] };
    }
    let count = count.max(2);
    let step = nice_step((max - min) / (count - 1) as f64);
    let mut v = (min / step).ceil();
    let mut out = Vec::new();
    while v * step <= max + step * 1e-9 && out.len() < 1000 {
        out.push(v * step);
        v += 1.0;
    }
    out
}

/// Expand `[min, max]` outward to the nearest "nice" tick multiples.
pub fn nice_bounds(min: f64, max: f64, count: usize) -> (f64, f64) {
    if !min.is_finite() || !max.is_finite() {
        return (0.0, 1.0);
    }
    if max <= min {
        return (min - 1.0, max + 1.0);
    }
    let step = nice_step((max - min) / (count.max(2) - 1) as f64);
    ((min / step).floor() * step, (max / step).ceil() * step)
}

/// Step between the first two ticks (or 1.0).
pub fn tick_step(ticks: &[f64]) -> f64 {
    if ticks.len() >= 2 {
        ticks[1] - ticks[0]
    } else {
        1.0
    }
}

/// Format a tick value with just enough decimals for the given step.
pub fn format_tick(v: f64, step: f64) -> String {
    let decimals = if !step.is_finite() || step <= 0.0 {
        0
    } else {
        (0..=8usize)
            .find(|d| {
                let x = step * 10f64.powi(*d as i32);
                (x - x.round()).abs() < 1e-6 * x.max(1.0)
            })
            .unwrap_or(8)
    };
    let s = format!("{v:.decimals$}");
    if s.starts_with('-') && s[1..].chars().all(|c| c == '0' || c == '.') {
        s[1..].to_string()
    } else {
        s
    }
}

/// Compact human format: `1.5k`, `2.3M`, `4B`.
pub fn format_compact(v: f64) -> String {
    let a = v.abs();
    let (d, suffix) = if a >= 1e9 {
        (1e9, "B")
    } else if a >= 1e6 {
        (1e6, "M")
    } else if a >= 1e3 {
        (1e3, "k")
    } else {
        (1.0, "")
    };
    let x = v / d;
    if suffix.is_empty() {
        if (x - x.round()).abs() < 1e-9 {
            format!("{}", x.round() as i64)
        } else {
            format!("{x:.2}")
        }
    } else if (x - x.round()).abs() < 0.05 {
        format!("{}{suffix}", x.round() as i64)
    } else {
        format!("{x:.1}{suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_is_nice() {
        assert_eq!(nice_step(0.7), 1.0);
        assert_eq!(nice_step(1.5), 2.0);
        assert_eq!(nice_step(2.2), 2.5);
        assert_eq!(nice_step(3.0), 5.0);
        assert_eq!(nice_step(7.0), 10.0);
        assert_eq!(nice_step(0.0), 1.0);
        assert_eq!(nice_step(f64::NAN), 1.0);
    }

    #[test]
    fn ticks_cover_range() {
        let t = nice_ticks(0.0, 100.0, 5);
        assert_eq!(t, vec![0.0, 25.0, 50.0, 75.0, 100.0]);
        let t = nice_ticks(0.0, 1.0, 5);
        assert_eq!(t.len(), 5);
        assert!((t[1] - 0.25).abs() < 1e-12);
    }

    #[test]
    fn ticks_degenerate() {
        assert_eq!(nice_ticks(3.0, 3.0, 5), vec![3.0]);
        assert!(nice_ticks(f64::NAN, 1.0, 5).is_empty());
    }

    #[test]
    fn bounds_expand() {
        let (lo, hi) = nice_bounds(3.2, 97.5, 5);
        assert!(lo <= 3.2 && hi >= 97.5);
        assert_eq!(nice_bounds(5.0, 5.0, 5), (4.0, 6.0));
    }

    #[test]
    fn tick_formatting() {
        assert_eq!(format_tick(20.0, 20.0), "20");
        assert_eq!(format_tick(2.5, 2.5), "2.5");
        assert_eq!(format_tick(0.25, 0.05), "0.25");
        assert_eq!(format_tick(-0.0, 0.1), "0.0");
        assert_eq!(tick_step(&[1.0, 3.0, 5.0]), 2.0);
        assert_eq!(tick_step(&[1.0]), 1.0);
    }

    #[test]
    fn compact_formatting() {
        assert_eq!(format_compact(999.0), "999");
        assert_eq!(format_compact(1500.0), "1.5k");
        assert_eq!(format_compact(2_000_000.0), "2M");
        assert_eq!(format_compact(3_400_000_000.0), "3.4B");
        assert_eq!(format_compact(1.25), "1.25");
    }
}
