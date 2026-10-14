#![cfg(feature = "serde")]

use ratatui::style::Color;
#[cfg(feature = "theme")]
use tui_chart::Theme;
use tui_chart::{Annotation, BarGroup, Bin, BoxStats, Candle, Task, TradeMarker, TradeSide};

fn rt<T>(v: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(v).unwrap()).unwrap()
}

#[test]
fn candle_roundtrip() {
    let c = Candle::new(1.0, 2.0, 0.5, 1.5).volume(10.0).label("x");
    assert_eq!(rt(&c), c);
}

#[test]
fn bar_group_roundtrip() {
    let g = BarGroup {
        label: "a".into(),
        values: vec![1.0, 2.0],
        bar_colors: vec![Some(Color::Red), None],
    };
    assert_eq!(rt(&g), g);
}

#[test]
fn bin_task_marker_roundtrip() {
    let b = Bin {
        start: 0.0,
        end: 1.0,
        count: 3,
    };
    assert_eq!(rt(&b), b);
    let t = Task::new("t", 0.0, 2.0);
    assert_eq!(rt(&t), t);
    let m = TradeMarker::new(1, 2.0, TradeSide::Buy).label("b");
    assert_eq!(rt(&m), m);
}

#[test]
fn annotation_boxstats_roundtrip() {
    let a = Annotation::hline(3.0);
    assert_eq!(rt(&a), a);
    let s = BoxStats {
        min: 0.0,
        q1: 1.0,
        median: 2.0,
        q3: 3.0,
        max: 4.0,
        outliers: vec![9.0],
    };
    assert_eq!(rt(&s), s);
}

#[cfg(feature = "theme")]
#[test]
fn theme_roundtrip() {
    for t in Theme::all() {
        assert_eq!(rt(&t), t);
    }
}
