//! ChartStack demo: several charts laid out in one stack with weighted panes.
//! Keys: o orientation · w weights · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run, Rng};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::Color,
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{
    Axis, BarChart, BarGroup, ChartStack, LabelPosition, LineChart, LineMode, Series,
    StackDirection, Theme,
};

const WEIGHTS: [[u16; 3]; 3] = [[3, 2, 2], [1, 1, 1], [5, 2, 1]];

struct State {
    theme: Theme,
    vertical: bool,
    weights: usize,
    price: Vec<(f64, f64)>,
    trend: Vec<(f64, f64)>,
    volume: Vec<f64>,
}

fn main() -> anyhow::Result<()> {
    let mut rng = Rng::new(11);
    let mut last = 100.0;
    let mut price = vec![];
    let mut trend = vec![];
    let mut volume = vec![];
    for i in 0..60 {
        last += rng.gauss() * 1.5;
        price.push((i as f64, last));
        trend.push((i as f64, 100.0 + i as f64 * 0.2));
        volume.push(10.0 + rng.f64() * 40.0);
    }
    run(
        State {
            theme: Theme::default(),
            vertical: true,
            weights: 0,
            price,
            trend,
            volume,
        },
        Duration::from_millis(100),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let dir = if s.vertical {
                StackDirection::Vertical
            } else {
                StackDirection::Horizontal
            };
            let panes = ChartStack::new(dir).weights(WEIGHTS[s.weights]).split(main);

            let area = LineChart::new(vec![Series::new("price")
                .data(s.price.clone())
                .color(Color::Cyan)])
            .mode(LineMode::Area)
            .y_axis(Axis::new().grid(true))
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Price "));
            f.render_widget(&area, panes[0]);

            let trend = LineChart::new(vec![
                Series::new("price")
                    .data(s.price.clone())
                    .color(Color::Cyan),
                Series::new("trend")
                    .data(s.trend.clone())
                    .color(Color::Yellow),
            ])
            .mode(LineMode::Step)
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Price vs trend "));
            f.render_widget(&trend, panes[1]);

            let vols: Vec<BarGroup> = s
                .volume
                .iter()
                .take(30)
                .map(|v| BarGroup::new("", [*v]))
                .collect();
            let bars = BarChart::new(vols)
                .show_values(false)
                .label_position(LabelPosition::Outside)
                .colors(vec![Color::Magenta])
                .theme(s.theme.clone())
                .block(Block::bordered().title(" Volume "));
            f.render_widget(&bars, panes[2]);

            help(f, bar, " o orientation · w weights · t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('o') => s.vertical = !s.vertical,
                KeyCode::Char('w') => s.weights = (s.weights + 1) % WEIGHTS.len(),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
