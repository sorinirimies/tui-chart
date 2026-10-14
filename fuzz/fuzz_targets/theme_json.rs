#![no_main]
use libfuzzer_sys::fuzz_target;
use tui_chart::*;

fuzz_target!(|data: &[u8]| {
    if let Ok(t) = serde_json::from_slice::<Theme>(data) {
        let s = serde_json::to_string(&t).expect("serialize theme");
        let back: Theme = serde_json::from_str(&s).expect("roundtrip theme");
        assert_eq!(serde_json::to_string(&back).unwrap(), s);
        let _ = (t.color(7), t.base_style(), t.gradient(), t.next(), t.prev());
    }
    let _ = serde_json::from_slice::<Vec<Candle>>(data);
    let _ = serde_json::from_slice::<Vec<BarGroup>>(data);
    let _ = serde_json::from_slice::<Vec<Annotation>>(data);
    let _ = serde_json::from_slice::<Vec<Task>>(data);
    let _ = serde_json::from_slice::<Vec<RadarSeries>>(data);
    let _ = serde_json::from_slice::<Vec<TreemapItem>>(data);
    let _ = serde_json::from_slice::<Gradient>(data);
    if let Ok(c) = serde_json::from_slice::<Candle>(data) {
        let s = serde_json::to_string(&c).unwrap();
        let _: Candle = serde_json::from_str(&s).unwrap();
    }
});
