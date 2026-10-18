//! Golden (snapshot) tests: every widget rendered from deterministic data at
//! two sizes is compared against `tests/golden/<widget>_<w>x<h>.txt`.
//!
//! Review changes, then re-bless with `UPDATE_GOLDEN=1 cargo test --test golden_test`.

mod common;
use common::*;
use ratatui::{buffer::Buffer, layout::Rect};
use std::{fs, path::PathBuf};

const SIZES: [(u16, u16); 2] = [(40, 12), (80, 24)];

fn dump(buf: &Buffer, area: Rect) -> String {
    let mut out = String::new();
    for y in area.top()..area.bottom() {
        let row: String = (area.left()..area.right())
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect();
        out.push_str(row.trim_end());
        out.push('\n');
    }
    out
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn path(name: &str, w: u16, h: u16) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!("{}_{w}x{h}.txt", sanitize(name)))
}

#[test]
fn widgets_match_golden_output() {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    let mut failures = Vec::new();
    let mut checked = 0;
    for (name, draw) in all_widgets(Data::Normal, None) {
        for (w, h) in SIZES {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            draw(area, &mut buf);
            let got = dump(&buf, area);
            let file = path(name, w, h);
            if update {
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, &got).unwrap();
                continue;
            }
            checked += 1;
            match fs::read_to_string(&file) {
                Ok(want) if want == got => {}
                Ok(_) => failures.push(format!("{name} {w}x{h}: output changed")),
                Err(_) => failures.push(format!("{name} {w}x{h}: no golden file")),
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{}\n\nre-bless with: UPDATE_GOLDEN=1 cargo test --test golden_test",
        failures.join("\n")
    );
    assert!(
        update || checked >= 48,
        "too few goldens checked: {checked}"
    );
}

#[test]
fn rendering_is_deterministic() {
    for (name, draw) in all_widgets(Data::Normal, None) {
        let area = Rect::new(0, 0, 60, 18);
        let (mut a, mut b) = (Buffer::empty(area), Buffer::empty(area));
        draw(area, &mut a);
        draw(area, &mut b);
        assert_eq!(a, b, "{name} is not deterministic");
    }
}
