//! Mouse interactivity: hover tooltips, click to pin, click a legend entry to
//! hide a series. Works on every chart through `HitMap` + `Interaction`.
//!
//! Wheel (or + / -) zooms the line and candle charts, [ ] pan.
//! Keys: Tab next chart · arrows / hjkl move the pointer (no mouse needed) · space click · r reset · q quit

#[path = "common/mod.rs"]
mod common;

use common::Rng;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEventKind,
    },
    execute,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
    Frame,
};
use std::cell::{Cell, RefCell};
use tui_chart::*;

const PAGES: [&str; 11] = [
    "Line",
    "Bar",
    "Candles",
    "Stacked area",
    "Radar",
    "Treemap",
    "Heatmap",
    "Funnel",
    "Pie",
    "Radial",
    "Instruments",
];

struct App {
    page: usize,
    ui: Interaction,
    view: View,
    plot: Cell<Rect>,
    hits: RefCell<HitMap>,
    data: Data,
    status: String,
}

struct Data {
    lines: Vec<Vec<(f64, f64)>>,
    candles: Vec<Candle>,
    heat: Vec<Vec<f64>>,
}

fn data() -> Data {
    let mut rng = Rng::new(11);
    let lines = (0..3)
        .map(|s| {
            let mut v = 10.0 + s as f64 * 6.0;
            (0..40)
                .map(|i| {
                    v += rng.gauss() * 1.5;
                    (f64::from(i), v)
                })
                .collect()
        })
        .collect();
    let mut p = 100.0;
    let candles = (0..40)
        .map(|_| {
            let o = p;
            p += rng.gauss() * 2.0;
            let c = p;
            Candle::new(o, o.max(c) + rng.f64() * 2.0, o.min(c) - rng.f64() * 2.0, c)
        })
        .collect();
    let heat = (0..7)
        .map(|_| (0..12).map(|_| rng.f64()).collect())
        .collect();
    Data {
        lines,
        candles,
        heat,
    }
}

/// Render the current page into `buf` and capture its hit map.
fn draw_page(app: &App, area: Rect, buf: &mut Buffer) -> HitMap {
    let hidden = app.ui.hidden();
    let block = Block::bordered().title(format!(" {} ", PAGES[app.page]));
    let d = &app.data;
    match app.page {
        0 => {
            let names = ["alpha", "beta", "gamma"];
            let series = d
                .lines
                .iter()
                .zip(names)
                .map(|(v, n)| Series::new(n).data(v.iter().copied()))
                .collect();
            let (x0, x1) = app.view.range(0.0, 39.0);
            HitMap::capture(
                LineChart::new(series)
                    .x_axis(Axis::new().bounds(x0, x1))
                    .y_axis(Axis::new().bounds(0.0, 35.0))
                    .hidden(hidden)
                    .block(block),
                area,
                buf,
            )
        }
        1 => HitMap::capture(
            BarChart::new(
                ["Mon", "Tue", "Wed", "Thu", "Fri"]
                    .iter()
                    .enumerate()
                    .map(|(i, l)| {
                        BarGroup::new(
                            *l,
                            [3.0 + i as f64, 8.0 - i as f64, 4.0 + (i % 2) as f64 * 3.0],
                        )
                    })
                    .collect(),
            )
            .series_names(["north", "south", "east"])
            .hidden(hidden)
            .block(block),
            area,
            buf,
        ),
        2 => HitMap::capture(
            CandlestickChart::new(app.view.slice(&d.candles).to_vec()).block(block),
            area,
            buf,
        ),
        3 => HitMap::capture(
            StackedAreaChart::new([
                AreaLayer::new(
                    "web",
                    (0..30).map(|i| 5.0 + (f64::from(i) * 0.4).sin() * 2.0),
                ),
                AreaLayer::new(
                    "api",
                    (0..30).map(|i| 4.0 + (f64::from(i) * 0.3).cos() * 2.0),
                ),
                AreaLayer::new("batch", (0..30).map(|i| 3.0 + f64::from(i % 7))),
            ])
            .hidden(hidden)
            .block(block),
            area,
            buf,
        ),
        4 => HitMap::capture(
            RadarChart::new(["Speed", "Power", "Armor", "Magic", "Range"])
                .series(RadarSeries::new("Knight", [6.0, 8.0, 9.0, 2.0, 3.0]))
                .series(RadarSeries::new("Mage", [3.0, 4.0, 2.0, 9.0, 7.0]))
                .hidden(hidden)
                .block(block),
            area,
            buf,
        ),
        5 => HitMap::capture(
            Treemap::new([
                TreemapItem::new("rust", 40.0),
                TreemapItem::new("go", 22.0),
                TreemapItem::new("zig", 9.0),
                TreemapItem::new("c", 18.0),
                TreemapItem::new("nim", 5.0),
            ])
            .block(block),
            area,
            buf,
        ),
        6 => HitMap::capture(Heatmap::new(d.heat.clone()).block(block), area, buf),
        7 => HitMap::capture(
            Funnel::new([
                FunnelStage::new("Visits", 10_000.0),
                FunnelStage::new("Signups", 2_400.0),
                FunnelStage::new("Trials", 900.0),
                FunnelStage::new("Paid", 310.0),
            ])
            .hidden(hidden)
            .block(block),
            area,
            buf,
        ),
        8 => HitMap::capture(
            Pie::new(
                PieChart::new(vec![
                    PieSlice::new("Rust", 45.0, Color::Red),
                    PieSlice::new("Go", 30.0, Color::Blue),
                    PieSlice::new("Zig", 15.0, Color::Green),
                    PieSlice::new("C", 10.0, Color::Yellow),
                ])
                .hole(0.45)
                .block(block),
            )
            .hidden(hidden),
            area,
            buf,
        ),
        9 => HitMap::capture(
            RadialBar::new([
                RadialRing::new("Move", 420.0, 600.0),
                RadialRing::new("Exercise", 25.0, 30.0),
                RadialRing::new("Stand", 9.0, 12.0),
            ])
            .hidden(hidden)
            .block(block),
            area,
            buf,
        ),
        _ => HitMap::capture(
            Spectrum::new((0..24).map(|i| jitter(0.9, i, 3.0)).collect::<Vec<_>>())
                .peaks(
                    (0..24)
                        .map(|i| jitter(0.9, i, 3.0).max(0.4))
                        .collect::<Vec<_>>(),
                )
                .block(block),
            area,
            buf,
        ),
    }
}

impl App {
    /// Zoom the x axis around column `x` (pointer column if known).
    fn scroll(&mut self, fallback: u16, steps: i32) {
        let x = self.ui.position().map_or(fallback, |p| p.0);
        self.view.on_scroll(x, self.plot.get(), steps);
    }

    /// Move the pointer with the keyboard (starts mid-screen).
    fn nudge(&mut self, dx: i32, dy: i32, size: (u16, u16)) {
        let (x, y) = self.ui.position().unwrap_or((size.0 / 3, size.1 / 3));
        let nx = (i32::from(x) + dx).clamp(0, i32::from(size.0) - 1) as u16;
        let ny = (i32::from(y) + dy).clamp(0, i32::from(size.1) - 1) as u16;
        self.ui.on_move(nx, ny);
    }
}

fn ui(f: &mut Frame, app: &App) {
    let [main, bar] = Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).areas(f.area());
    app.plot.set(main);
    let hits = draw_page(app, main, f.buffer_mut());
    app.ui.render_tooltip(&hits, f.buffer_mut(), main);
    if let Some((x, y)) = app.ui.position() {
        if let Some(c) = f.buffer_mut().cell_mut((x, y)) {
            c.set_style(Style::default().add_modifier(ratatui::style::Modifier::REVERSED));
        }
    }
    *app.hits.borrow_mut() = hits;
    f.render_widget(
        Paragraph::new(format!(
            " {}\n Tab next · mouse/arrows hover · click/space pin · legend toggles · wheel/+- zoom · [ ] pan · r reset · q quit",
            app.status
        ))
        .style(Style::default().fg(Color::DarkGray)),
        bar,
    );
}

fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableMouseCapture)?;
    let mut app = App {
        page: 0,
        ui: Interaction::default(),
        view: View::default(),
        plot: Cell::new(Rect::default()),
        hits: RefCell::new(HitMap::default()),
        data: data(),
        status: String::new(),
    };
    let result = (|| -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| ui(f, &app))?;
            let size = terminal.size().map(|s| (s.width, s.height))?;
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Tab => {
                        app.page = (app.page + 1) % PAGES.len();
                        {
                            app.ui.reset();
                            app.view.reset();
                        }
                    }
                    KeyCode::BackTab => {
                        app.page = (app.page + PAGES.len() - 1) % PAGES.len();
                        {
                            app.ui.reset();
                            app.view.reset();
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => app.nudge(-2, 0, size),
                    KeyCode::Right | KeyCode::Char('l') => app.nudge(2, 0, size),
                    KeyCode::Up | KeyCode::Char('k') => app.nudge(0, -1, size),
                    KeyCode::Down | KeyCode::Char('j') => app.nudge(0, 1, size),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        let (x, y) = app.ui.position().unwrap_or((0, 0));
                        let click = app.ui.on_click(x, y, &app.hits.borrow());
                        app.status = format!("{click:?}");
                    }
                    KeyCode::Char('+' | '=') => app.scroll(30, 1),
                    KeyCode::Char('-') => app.scroll(30, -1),
                    KeyCode::Char('[') => app.view.pan(-0.25),
                    KeyCode::Char(']') => app.view.pan(0.25),
                    KeyCode::Char('r') => {
                        app.ui.reset();
                        app.view.reset();
                    }
                    _ => {}
                },
                Event::Mouse(m) => match m.kind {
                    MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                        app.ui.on_move(m.column, m.row);
                    }
                    MouseEventKind::ScrollUp => {
                        app.ui.on_move(m.column, m.row);
                        app.scroll(m.column, 1);
                    }
                    MouseEventKind::ScrollDown => {
                        app.ui.on_move(m.column, m.row);
                        app.scroll(m.column, -1);
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        let click = app.ui.on_click(m.column, m.row, &app.hits.borrow());
                        app.status = format!("{click:?}");
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    })();
    execute!(std::io::stdout(), DisableMouseCapture)?;
    ratatui::restore();
    result
}
