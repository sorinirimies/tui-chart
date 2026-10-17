//! Bullet chart demo. Keys: up/down adjust first value · t/T theme · q quit

#[path = "common/mod.rs"]
mod common;

use common::{help, run};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};
use std::time::Duration;
use tui_chart::{Bullet, BulletItem, Theme};

struct State {
    theme: Theme,
    revenue: f64,
}

fn main() -> anyhow::Result<()> {
    run(
        State {
            theme: Theme::default(),
            revenue: 270.0,
        },
        Duration::from_millis(200),
        |f, s| {
            let [main, bar] =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
            let chart = Bullet::new([
                BulletItem::new("Revenue", s.revenue)
                    .target(300.0)
                    .ranges([150.0, 250.0, 350.0]),
                BulletItem::new("Profit", 22.0)
                    .target(27.0)
                    .ranges([15.0, 25.0, 35.0]),
                BulletItem::new("Orders", 1500.0)
                    .target(2000.0)
                    .ranges([1000.0, 2000.0, 3000.0]),
                BulletItem::new("Satisfaction", 3.9)
                    .target(4.5)
                    .ranges([3.0, 4.0, 5.0]),
            ])
            .theme(s.theme.clone())
            .block(Block::bordered().title(" KPIs "));
            f.render_widget(&chart, main);
            help(f, bar, " up/down value · t/T theme · q quit");
        },
        |s, key| {
            match key {
                KeyCode::Char('t') => s.theme = s.theme.next(),
                KeyCode::Char('T') => s.theme = s.theme.prev(),
                KeyCode::Up => s.revenue = (s.revenue + 20.0).min(350.0),
                KeyCode::Down => s.revenue = (s.revenue - 20.0).max(0.0),
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            }
            false
        },
        |_| {},
    )
}
