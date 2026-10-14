//! Streaming multi-pane demo: rolling line + histogram with a time axis.
//! Keys: space pause · +/- speed · q quit

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
    time_hms, Axis, BarChart, BarGroup, ChartStack, LabelPosition, LineChart, LineMode, Stream,
};

struct State {
    rng: Rng,
    price: Stream,
    volume: Vec<f64>,
    last: f64,
    paused: bool,
}

fn main() -> anyhow::Result<()> {
    let start = 1_700_000_000.0;
    let mut price = Stream::new(120).x_start(start);
    price.push(100.0);
    run(
        State {
            rng: Rng::new(7),
            price,
            volume: vec![],
            last: 100.0,
            paused: false,
        },
        Duration::from_millis(100),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
            let panes = ChartStack::vertical().weights([3, 1]).split(main);
            let chart = LineChart::new(vec![s.price.series("price").color(Color::Cyan)])
                .mode(LineMode::Area)
                .x_axis(Axis::new().formatter(time_hms).ticks(5))
                .y_axis(Axis::new().grid(true))
                .block(Block::bordered().title(" Live price (UTC) "));
            f.render_widget(&chart, panes[0]);
            let vols: Vec<BarGroup> = s
                .volume
                .iter()
                .rev()
                .take(24)
                .rev()
                .map(|v| BarGroup::new("", [*v]))
                .collect();
            let bars = BarChart::new(vols)
                .show_values(false)
                .label_position(LabelPosition::Outside)
                .colors(vec![Color::Magenta])
                .block(Block::bordered().title(" Volume "));
            f.render_widget(&bars, panes[1]);
            help(f, bar, " space pause · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char(' ') => s.paused = !s.paused,
                _ => {}
            }
            false
        },
        |s| {
            if s.paused {
                return;
            }
            s.last += s.rng.gauss() * 0.8;
            s.price.push(s.last);
            s.volume.push(10.0 + s.rng.f64() * 40.0);
            if s.volume.len() > 200 {
                s.volume.remove(0);
            }
        },
    )
}
