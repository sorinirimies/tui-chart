//! Robustness matrix: every public widget × area × data kind must not panic
//! and must not write outside its area.

mod common;
use common::*;
use ratatui::layout::Rect;

#[test]
fn no_panic_no_outside_writes_matrix() {
    let mut n = 0;
    let mut bad: Vec<String> = Vec::new();
    for d in DATAS {
        for (name, draw) in all_widgets(d, None) {
            for (w, h) in AREAS {
                // Offset areas catch absolute/relative coordinate mix-ups.
                for (ox, oy) in [(0u16, 0u16), (2, 1)] {
                    let area = Rect::new(ox, oy, w, h);
                    let (buf, full) = render_padded(&draw, area);
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        assert_no_outside_writes(&format!("{name} {d:?}"), &buf, full, area)
                    }));
                    if r.is_err() {
                        bad.push(format!("{name} {d:?} {area:?}"));
                    }
                    n += 1;
                }
            }
        }
    }
    bad.dedup();
    assert!(bad.is_empty(), "outside writes:\n{}", bad.join("\n"));
    assert!(n > 900, "matrix too small: {n}");
}

#[test]
fn area_larger_than_buffer_is_clipped_or_safe() {
    // Rendering into an area that exceeds the buffer must not panic.
    use ratatui::buffer::Buffer;
    for d in DATAS {
        for (name, draw) in all_widgets(d, None) {
            let mut buf = Buffer::empty(Rect::new(0, 0, 20, 8));
            let area = Rect::new(0, 0, 20, 8);
            draw(area, &mut buf);
            let _ = name;
        }
    }
}

#[test]
fn rendering_twice_is_deterministic() {
    for d in DATAS {
        for (name, draw) in all_widgets(d, None) {
            let area = Rect::new(0, 0, 40, 12);
            let (a, fa) = render_padded(&draw, area);
            let (b, _) = render_padded(&draw, area);
            assert_eq!(a, b, "{name} {d:?} not deterministic");
            let _ = fa;
        }
    }
}
