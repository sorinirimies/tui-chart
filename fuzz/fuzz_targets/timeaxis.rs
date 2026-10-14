#![no_main]
use libfuzzer_sys::fuzz_target;
use tui_chart::*;

fuzz_target!(|d: (f64, f64, u8, i64, u32, u32)| {
    let (a, b, n, z, m, day) = d;
    let _ = (time_hms(a), time_hm(a), date_ymd(a), date_md(a), datetime(a));
    let _ = time_nice_step(a, n as usize);
    let ticks = time_ticks(a, b, n as usize);
    assert!(ticks.len() <= 1_000_000, "runaway tick count {}", ticks.len());
    let _ = civil_from_days(z / 4);
    let _ = days_from_civil((z / 4) % 1_000_000_000, m, day);
});
