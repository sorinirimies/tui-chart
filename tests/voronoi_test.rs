//! Integration tests for `VoronoiChart` and `VoronoiTreemap`.

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_chart::*;

fn sites() -> Vec<VoronoiSite> {
    vec![
        VoronoiSite::new("west", 0.0, 0.0)
            .value(1.0)
            .weight(2.0)
            .category("a"),
        VoronoiSite::new("east", 10.0, 0.0)
            .value(9.0)
            .weight(1.0)
            .category("b"),
        VoronoiSite::new("north", 5.0, 8.0).value(5.0).category("b"),
    ]
}

fn capture(w: impl Widget) -> HitMap {
    let area = Rect::new(0, 0, 60, 20);
    let mut buf = Buffer::empty(area);
    HitMap::capture(w, area, &mut buf)
}

#[test]
fn tooltip_lists_value_category_area_and_neighbours() {
    let h = capture(VoronoiChart::new(sites()));
    let hit = h
        .hits()
        .iter()
        .find(|x| x.title == "east")
        .expect("east cell");
    let text = hit.lines.join("\n");
    assert!(text.contains("value 9"), "{text}");
    assert!(text.contains("category b"), "{text}");
    assert!(text.contains("area "), "{text}");
    assert!(text.contains("neighbours"), "{text}");
}

#[test]
fn every_site_gets_a_cell() {
    let h = capture(VoronoiChart::new(sites()));
    let mut titles: Vec<&str> = h.hits().iter().map(|x| x.title.as_str()).collect();
    titles.sort();
    titles.dedup();
    assert_eq!(titles, ["east", "north", "west"]);
}

#[test]
fn probe_is_not_a_hit_but_changes_neighbours() {
    let plain = capture(VoronoiChart::new(sites()));
    let probed = capture(VoronoiChart::new(sites()).probe((5.0, 2.0)));
    assert_eq!(plain.hits().len(), probed.hits().len());
    assert!(
        probed
            .hits()
            .iter()
            .any(|h| h.lines.iter().any(|l| l.contains("probe")))
    );
}

#[test]
fn treemap_hit_titles_carry_the_group_path() {
    let t = VoronoiTreemap::new([
        TreemapItem::new("G", 0.0)
            .children([TreemapItem::new("a", 3.0), TreemapItem::new("b", 2.0)]),
        TreemapItem::new("c", 4.0),
    ]);
    let h = capture(t);
    let titles: Vec<&str> = h.hits().iter().map(|x| x.title.as_str()).collect();
    assert!(titles.contains(&"G › a") && titles.contains(&"G › b") && titles.contains(&"c"));
}

#[test]
fn monochrome_turns_cell_fills_into_shades() {
    let area = Rect::new(0, 0, 40, 12);
    let mut buf = Buffer::empty(area);
    Monochrome::new(VoronoiChart::new(sites()).borders(false)).render(area, &mut buf);
    assert!(
        buf.content()
            .iter()
            .any(|c| matches!(c.symbol(), "░" | "▒" | "▓" | "█"))
    );
    let mut buf = Buffer::empty(area);
    Monochrome::new(VoronoiTreemap::new([
        TreemapItem::new("a", 1.0),
        TreemapItem::new("b", 2.0),
    ]))
    .render(area, &mut buf);
    assert!(
        buf.content()
            .iter()
            .any(|c| matches!(c.symbol(), "░" | "▒" | "▓" | "█"))
    );
}

#[test]
fn rendering_by_reference_matches_owned() {
    let area = Rect::new(0, 0, 40, 12);
    let c = VoronoiChart::new(sites()).relax(2).delaunay(true);
    let (mut a, mut b) = (Buffer::empty(area), Buffer::empty(area));
    (&c).render(area, &mut a);
    c.render(area, &mut b);
    assert_eq!(a, b);
}

#[test]
fn treemap_total_area_matches_for_many_items() {
    // 12 equal items: every item should get a visible share of the plot
    let t = VoronoiTreemap::new((0..12).map(|i| TreemapItem::new(format!("i{i}"), 1.0)))
        .labels(false)
        .borders(false);
    let h = capture(t);
    assert_eq!(
        h.hits().iter().filter(|x| x.kind == HitKind::Point).count(),
        12
    );
}
