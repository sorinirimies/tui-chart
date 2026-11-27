//! Voronoi chart demo.
//! Keys: v scene · m metric · w weighted · f farthest · r relax · d delaunay ·
//! s shade · a animate · n select · arrows move probe · l labels · b borders · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{Rng, help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Gradient, Theme, VoronoiChart, VoronoiMetric, VoronoiSite, voronoi_stipple};

#[derive(Clone, Copy, PartialEq)]
enum Scene {
    Classic,
    Heatmap,
    Territories,
    Knn,
    Mosaic,
    Ring,
}

impl Scene {
    const ALL: [Scene; 6] = [
        Scene::Classic,
        Scene::Heatmap,
        Scene::Territories,
        Scene::Knn,
        Scene::Mosaic,
        Scene::Ring,
    ];
    fn title(self) -> &'static str {
        match self {
            Scene::Classic => "Service areas",
            Scene::Heatmap => "Sensor coverage (value heatmap)",
            Scene::Territories => "Territories (categories, merged borders)",
            Scene::Knn => "k-NN classifier (k = 3)",
            Scene::Mosaic => "Stipple mosaic (density → sites)",
            Scene::Ring => "Voronoi pie",
        }
    }
}

struct State {
    theme: Theme,
    scene: usize,
    metric: VoronoiMetric,
    weighted: bool,
    farthest: bool,
    relax: usize,
    delaunay: bool,
    shade: bool,
    animate: bool,
    phase: f64,
    labels: bool,
    borders: bool,
    select: Option<usize>,
    probe: (f64, f64),
    sites: Vec<(String, f64, f64, f64, usize)>,
    mosaic: Vec<VoronoiSite>,
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(11);
    let sites = (0..16)
        .map(|i| {
            let (x, y) = (rng.f64() * 100.0, rng.f64() * 60.0);
            // spatially clustered categories so territories look natural
            let cat = usize::from(x > 50.0) + 2 * usize::from(y > 30.0);
            (format!("S{i}"), x, y, 1.0 + rng.f64() * 9.0, cat)
        })
        .collect();
    // a soft blob density for the mosaic
    let density: Vec<Vec<f64>> = (0..30)
        .map(|r| {
            (0..80)
                .map(|c| {
                    let (dx, dy) = ((c as f64 - 40.0) / 20.0, (r as f64 - 15.0) / 9.0);
                    (-(dx * dx + dy * dy)).exp() + 0.05
                })
                .collect()
        })
        .collect();
    let mosaic = voronoi_stipple(&density, 220, 6, 3);
    run(
        State {
            theme: Theme::default(),
            scene: 0,
            metric: VoronoiMetric::Euclidean,
            weighted: false,
            farthest: false,
            relax: 0,
            delaunay: false,
            shade: false,
            animate: false,
            phase: 0.0,
            labels: true,
            borders: true,
            select: None,
            probe: (50.0, 30.0),
            sites,
            mosaic,
        },
        Duration::from_millis(120),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let scene = Scene::ALL[s.scene];
            let cats = ["north-west", "north-east", "south-west", "south-east"];
            let mut chart = match scene {
                Scene::Ring => VoronoiChart::ring(
                    s.sites
                        .iter()
                        .take(6)
                        .map(|(n, _, _, w, _)| (n.clone(), *w)),
                ),
                Scene::Mosaic => VoronoiChart::new(s.mosaic.clone())
                    .gradient(Gradient::Heat)
                    .show_sites(false)
                    .show_labels(false)
                    .borders(false)
                    .square(false),
                _ => VoronoiChart::new(s.sites.iter().map(|(n, x, y, w, c)| {
                    let mut site = VoronoiSite::new(n.clone(), *x, *y)
                        .weight(*w)
                        .value(*w * 10.0);
                    if matches!(scene, Scene::Territories | Scene::Knn) {
                        site = site.category(cats[*c]);
                    }
                    site
                })),
            };
            if scene == Scene::Heatmap {
                chart = chart.gradient(Gradient::Viridis).axes(true);
            }
            if scene == Scene::Territories {
                chart = chart.merge_categories(true);
            }
            if scene == Scene::Knn {
                chart = chart.knn(3).show_labels(false);
            }
            if scene != Scene::Ring && scene != Scene::Mosaic {
                chart = chart
                    .metric(s.metric)
                    .weighted(s.weighted)
                    .farthest(s.farthest)
                    .relax(s.relax)
                    .delaunay(s.delaunay)
                    .shade_by_distance(s.shade)
                    .show_labels(s.labels && scene != Scene::Knn)
                    .borders(s.borders)
                    .phase(if s.animate { s.phase } else { 0.0 })
                    .probe(s.probe);
                if let Some(i) = s.select {
                    chart = chart.select(i % s.sites.len());
                }
            }
            f.render_widget(
                chart
                    .theme(s.theme.clone())
                    .block(Block::bordered().title(format!(" {} ", scene.title()))),
                main,
            );
            help(
                f,
                bar,
                " v scene · m metric · w weighted · f farthest · r relax · d delaunay · s shade · a animate · n select · arrows probe · l labels · b borders · t/T theme · q quit",
            );
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('v') => s.scene = (s.scene + 1) % Scene::ALL.len(),
                KeyCode::Char('m') => s.metric = s.metric.next(),
                KeyCode::Char('w') => s.weighted = !s.weighted,
                KeyCode::Char('f') => s.farthest = !s.farthest,
                KeyCode::Char('r') => s.relax = if s.relax >= 8 { 0 } else { s.relax + 2 },
                KeyCode::Char('d') => s.delaunay = !s.delaunay,
                KeyCode::Char('s') => s.shade = !s.shade,
                KeyCode::Char('a') => s.animate = !s.animate,
                KeyCode::Char('n') => s.select = Some(s.select.map_or(0, |i| i + 1)),
                KeyCode::Char('l') => s.labels = !s.labels,
                KeyCode::Char('b') => s.borders = !s.borders,
                KeyCode::Left => s.probe.0 = (s.probe.0 - 4.0).max(0.0),
                KeyCode::Right => s.probe.0 = (s.probe.0 + 4.0).min(100.0),
                KeyCode::Up => s.probe.1 = (s.probe.1 + 3.0).min(60.0),
                KeyCode::Down => s.probe.1 = (s.probe.1 - 3.0).max(0.0),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |s| {
            if s.animate {
                s.phase += 0.15;
            }
        },
    )
}
