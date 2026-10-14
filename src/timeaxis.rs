//! Time-aware axis helpers: unix-timestamp (seconds) formatting and tick steps.
//!
//! No external crates: dates use Howard Hinnant's civil-from-days algorithm
//! (proleptic Gregorian, UTC). The `fn(f64) -> String` formatters plug directly into
//! [`Axis::formatter`](crate::Axis::formatter).

/// Days since 1970-01-01 → `(year, month 1..=12, day 1..=31)`.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `(year, month, day)` → days since 1970-01-01 (inverse of [`civil_from_days`]).
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Split a timestamp into `(y, m, d, hh, mm, ss)`; non-finite maps to the epoch.
fn split(ts: f64) -> (i64, u32, u32, u32, u32, u32) {
    let s = if ts.is_finite() {
        ts.clamp(-1e15, 1e15).floor() as i64
    } else {
        0
    };
    let days = s.div_euclid(86_400);
    let rem = s.rem_euclid(86_400) as u32;
    let (y, m, d) = civil_from_days(days);
    (y, m, d, rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Format as `HH:MM:SS` (UTC).
pub fn time_hms(ts: f64) -> String {
    let (_, _, _, h, m, s) = split(ts);
    format!("{h:02}:{m:02}:{s:02}")
}

/// Format as `HH:MM` (UTC).
pub fn time_hm(ts: f64) -> String {
    let (_, _, _, h, m, _) = split(ts);
    format!("{h:02}:{m:02}")
}

/// Format as `YYYY-MM-DD` (UTC).
pub fn date_ymd(ts: f64) -> String {
    let (y, m, d, ..) = split(ts);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Format as `MM-DD` (UTC).
pub fn date_md(ts: f64) -> String {
    let (_, m, d, ..) = split(ts);
    format!("{m:02}-{d:02}")
}

/// Format as `YYYY-MM-DD HH:MM:SS` (UTC).
pub fn datetime(ts: f64) -> String {
    format!("{} {}", date_ymd(ts), time_hms(ts))
}

const FIXED_STEPS: [f64; 22] = [
    1.0,
    2.0,
    5.0,
    10.0,
    15.0,
    30.0,
    60.0,
    120.0,
    300.0,
    600.0,
    900.0,
    1800.0,
    3600.0,
    7200.0,
    10_800.0,
    21_600.0,
    43_200.0,
    86_400.0,
    172_800.0,
    604_800.0,
    1_209_600.0,
    2_592_000.0,
];
/// Calendar steps in months (1 month .. 10 years); above these, years are scaled by 10x.
const MONTH_STEPS: [u32; 6] = [1, 3, 6, 12, 24, 60];

/// A nice time-aware step in seconds for covering `span` with about `count` ticks
/// (1 s, 5 s, 1 min, 1 h, 1 day, 1 week, ... up to 30 days; beyond that month/year
/// steps are returned as 30.44-day multiples, see [`time_ticks`] for calendar alignment).
pub fn time_nice_step(span: f64, count: usize) -> f64 {
    let raw = if span.is_finite() && span > 0.0 {
        span / count.max(1) as f64
    } else {
        1.0
    };
    if let Some(&s) = FIXED_STEPS.iter().find(|&&s| s >= raw) {
        return s;
    }
    let month = 2_629_746.0;
    for m in MONTH_STEPS {
        if m as f64 * month >= raw {
            return m as f64 * month;
        }
    }
    let mut y = 10.0;
    loop {
        for k in [1.0, 2.0, 5.0] {
            if y * k * 12.0 * month >= raw {
                return y * k * 12.0 * month;
            }
        }
        y *= 10.0;
        if y > 1e9 {
            return raw;
        }
    }
}

/// Tick timestamps within `[min, max]`, about `count` of them. Sub-month steps
/// align to multiples of the step (days align to UTC midnight); month/year steps
/// align to calendar month/year starts.
pub fn time_ticks(min: f64, max: f64, count: usize) -> Vec<f64> {
    if !min.is_finite() || !max.is_finite() || max < min {
        return vec![];
    }
    let step = time_nice_step(max - min, count);
    if step < 2_592_000.0 + 1.0 {
        let first = (min / step).ceil();
        let n = ((max / step).floor() - first) as i64;
        return (0..=n.clamp(-1, 10_000))
            .map(|i| (first + i as f64) * step)
            .collect();
    }
    let months = (step / 2_629_746.0).round().max(1.0) as i64;
    let (y, m, ..) = split(min);
    let mut idx = y * 12 + (m as i64 - 1);
    idx = (idx + months - 1).div_euclid(months) * months;
    let mut out = vec![];
    while out.len() < 10_000 {
        let (yy, mm) = (idx.div_euclid(12), idx.rem_euclid(12) as u32 + 1);
        let t = days_from_civil(yy, mm, 1) as f64 * 86_400.0;
        if t > max {
            break;
        }
        if t >= min {
            out.push(t);
        }
        idx += months;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_roundtrip_and_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        for d in (-800_000..800_000).step_by(997) {
            let (y, m, dd) = civil_from_days(d);
            assert_eq!(days_from_civil(y, m, dd), d);
        }
    }

    #[test]
    fn formatters() {
        assert_eq!(time_hms(3661.0), "01:01:01");
        assert_eq!(time_hm(3661.0), "01:01");
        assert_eq!(date_ymd(1_709_164_800.0), "2024-02-29");
        assert_eq!(date_md(1_709_164_800.0), "02-29");
        assert_eq!(datetime(1_709_164_800.0 + 45_296.0), "2024-02-29 12:34:56");
        assert_eq!(date_ymd(-86_400.0), "1969-12-31");
        assert_eq!(time_hms(f64::NAN), "00:00:00");
        assert_eq!(time_hms(-1.0), "23:59:59");
        let _ = datetime(f64::INFINITY);
    }

    #[test]
    fn nice_steps() {
        assert_eq!(time_nice_step(10.0, 10), 1.0);
        assert_eq!(time_nice_step(3600.0, 6), 600.0);
        assert_eq!(time_nice_step(86_400.0, 6), 21_600.0);
        assert!(time_nice_step(1e10, 5) > 2_592_000.0);
        assert_eq!(time_nice_step(f64::NAN, 0), 1.0);
    }

    #[test]
    fn ticks_aligned_and_bounded() {
        let t = time_ticks(0.0, 3600.0, 6);
        assert_eq!(t.first(), Some(&0.0));
        assert!(t.iter().all(|&x| (0.0..=3600.0).contains(&x)));
        assert!(t.windows(2).all(|w| w[1] - w[0] == 600.0));
        let year = 365.0 * 86_400.0;
        let m = time_ticks(1_700_000_000.0, 1_700_000_000.0 + 3.0 * year, 6);
        assert!(m.len() >= 2);
        assert!(m.iter().all(|&x| date_ymd(x).ends_with("-01")));
        assert!(time_ticks(5.0, 1.0, 5).is_empty());
        assert!(time_ticks(f64::NAN, 1.0, 5).is_empty());
        let _ = time_ticks(7.0, 7.0, 5);
    }
}
