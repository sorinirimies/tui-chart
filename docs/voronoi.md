# Voronoi charts

Two widgets, one idea: split the plot into cells and paint each cell with the
colour of the site it belongs to. Cells are drawn at **half-block resolution**
(one dot per column, two per row), so edges are true bisectors as they look on
screen.

| Widget | Cell area means | Use for |
|--------|-----------------|---------|
| `VoronoiChart` | "closest to this site" | coverage, catchments, territories, classifiers, mosaics |
| `VoronoiTreemap` | the item's value | part-of-whole with organic cells instead of rectangles |

Run the demos: `cargo run --features theme --example voronoi` and
`--example voronoi_treemap`.

## `VoronoiChart`

```rust
use tui_chart::{Gradient, VoronoiChart, VoronoiSite};

let chart = VoronoiChart::new([
    VoronoiSite::new("north", 12.0, 40.0).value(8.0),
    VoronoiSite::new("harbour", 60.0, 10.0).value(3.0),
    VoronoiSite::new("airport", 85.0, 45.0).value(9.0),
])
.gradient(Gradient::Viridis)   // heatmap: colour by `value`
.axes(true)                    // scatter-style tick labels
.show_labels(true);
```

### Variants

| You want | Call |
|----------|------|
| Plain diagram, one colour per site | `VoronoiChart::new(sites)` |
| **Heatmap** (colour by measurement) | `VoronoiSite::value(v)` + `.gradient(Gradient::..)` |
| **Territory map** (merged borders) | `VoronoiSite::category("..")`, `.merge_categories(true)` (default) |
| **k-NN classifier regions** | categories + `.knn(k)` |
| Other distance | `.metric(VoronoiMetric::Manhattan / Chebyshev / Minkowski(p))` |
| **Farthest-site** diagram | `.farthest(true)` |
| Heavier site, bigger cell | `VoronoiSite::weight(w)` + `.weighted(true)` (power diagram) |
| Even, honeycomb cells (**centroidal**) | `.relax(n)` – n Lloyd steps |
| Neighbour graph | `.delaunay(true)` |
| Glowing centres, dark edges | `.shade_by_distance(true)` |
| Drifting sites (animation) | `.phase(t)` – advance `t` every tick |
| Cursor as a site | `.probe((x, y))` – a temporary extra site, drawn as `◆` |
| Highlight a cell and its neighbours | `.select(site_index)` |
| Scatter overlay look | `.axes(true)`, `.bounds(x0, x1, y0, y1)`, `.square(true)` |
| **Voronoi pie** | `VoronoiChart::ring([(label, value), ..])` (wedges are approximately proportional) |
| Disc clip for any diagram | `.clip_circle(true)` |
| **Stipple / mosaic** | `voronoi_stipple(&density, n, relax, seed)` → sites carrying the density as `value` |

Presentation: `.borders()`, `.border_color()`, `.show_sites()`, `.show_labels()`,
`.fill_dim()`, `.theme()`, `.base_style()`, `.block()`.

### Stipple mosaic

```rust
use tui_chart::{Gradient, VoronoiChart, voronoi_stipple};

let density: Vec<Vec<f64>> = /* rows × cols, row 0 on top */ vec![vec![1.0; 40]; 20];
let sites = voronoi_stipple(&density, 250, 6, /* seed */ 1);
let chart = VoronoiChart::new(sites)
    .gradient(Gradient::Heat)
    .show_sites(false)
    .borders(false);
```

Sites crowd where density is high (density-weighted Lloyd relaxation).
Deterministic for a seed.

### Interaction

`HitMap::capture` reports one hit per cell. The tooltip shows position, weight,
value, category, the **area share** and the **neighbouring cells with the share
of the border each takes**. Drive `.probe(..)` from the mouse position and
`.select(..)` from a click to build the "nearest-neighbour query" interaction;
see [interactivity.md](interactivity.md).

### Notes

* Cost is `O(width · height · sites)` per frame (plus `relax` times that), so
  prefer a few hundred sites at most; `relax` is capped at 60.
* Sites with non-finite coordinates are skipped; NaN weights/values count as
  missing. Degenerate sizes render nothing instead of panicking.
* `Monochrome` post-processing turns cell fills into shade glyphs as usual.

## `VoronoiTreemap`

Same data type as `Treemap`: `TreemapItem` with optional children.

```rust
use tui_chart::{TreemapItem, VoronoiTreemap};

let chart = VoronoiTreemap::new([
    TreemapItem::new("Systems", 0.0).children([
        TreemapItem::new("Rust", 40.0),
        TreemapItem::new("C", 25.0),
    ]),
    TreemapItem::new("Apps", 0.0).children([
        TreemapItem::new("Kotlin", 30.0),
        TreemapItem::new("Swift", 20.0),
    ]),
])
.seed(7);
```

* The plot is partitioned among the top-level items, then each region is
  partitioned among its children, recursively.
* Each partition is a weighted (power) diagram: sites move to their cell
  centroids while weights are tuned until every cell has its target area.
  Areas land within a few percent of the values (verified in the tests).
* Deterministic: same items + `.seed(n)` → same picture. Change the seed for a
  different arrangement.
* Group borders are bright, leaf borders dark. Leaves of a group share its hue
  (`TreemapItem::color` or the theme palette) in graded shades.
* Labels sit at the cell centroid and are cut to fit the cell.
* `.iterations(n)` caps the relaxation (default 40); large areas lower it
  automatically.
* Tooltips show the path (`Group › Item`), value, share of the total and area.
