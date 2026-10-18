//! Funnel demo. Keys: a taper/bars · p toggle percent · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Funnel, FunnelStage, Theme};

struct State {
    theme: Theme,
    percent: bool,
    taper: bool,
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            percent: true,
            taper: true,
        },
        Duration::from_millis(200),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let chart = Funnel::new([
                FunnelStage::new("Visitors", 12_000.0),
                FunnelStage::new("Sign-ups", 4_800.0),
                FunnelStage::new("Trials", 2_100.0),
                FunnelStage::new("Paying", 780.0),
                FunnelStage::new("Retained", 520.0),
            ])
            .show_percent(s.percent)
            .taper(s.taper)
            .theme(s.theme.clone())
            .block(Block::bordered().title(" Conversion funnel "));
            f.render_widget(&chart, main);
            help(f, bar, " a taper · p percent · t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Char('p') => s.percent = !s.percent,
                KeyCode::Char('a') => s.taper = !s.taper,
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
