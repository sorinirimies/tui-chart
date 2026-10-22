//! Comparison chart demo. Keys: l layout · up/down adjust the "after" values · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Comparison, ComparisonItem, ComparisonLayout, Theme};

struct State {
    theme: Theme,
    layout: ComparisonLayout,
    scale: f64,
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            layout: ComparisonLayout::Inline,
            scale: 0.5,
        },
        Duration::from_millis(200),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let chart = Comparison::new([
                ComparisonItem::new("mean ΔE", 3.2, 3.2 * s.scale),
                ComparisonItem::new("p95 ΔE (worst 5%)", 8.4, 8.4 * s.scale),
                ComparisonItem::new("over ΔE 7", 12.0, 12.0 * s.scale).unit("%"),
            ])
            .layout(s.layout)
            .legend("46 shipped", "286 in use")
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Gain "));
            f.render_widget(&chart, main);
            help(f, bar, " l layout · up/down value · t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('l') => {
                    s.layout = match s.layout {
                        ComparisonLayout::Inline => ComparisonLayout::Stacked,
                        ComparisonLayout::Stacked => ComparisonLayout::Inline,
                    }
                }
                KeyCode::Up => s.scale = (s.scale + 0.1).min(1.2),
                KeyCode::Down => s.scale = (s.scale - 0.1).max(0.0),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
