//! Theme × widget matrix: no panics, and explicit colors beat the theme.

mod common;
use common::*;
use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use std::collections::HashSet;
use tui_chart::*;

fn render(w: impl Widget, width: u16, height: u16) -> Buffer {
    let r = Rect::new(0, 0, width, height);
    let mut b = Buffer::empty(r);
    w.render(r, &mut b);
    b
}

fn colors(b: &Buffer) -> HashSet<Color> {
    b.content.iter().flat_map(|c| [c.fg, c.bg]).collect()
}

#[test]
fn every_theme_every_widget_renders() {
    let themes = Theme::all();
    assert!(themes.len() >= 8);
    for t in themes {
        for d in [Data::Empty, Data::Single, Data::Nasty, Data::Normal] {
            for (name, draw) in all_widgets(d, Some(t.clone())) {
                for (w, h) in [(0u16, 0u16), (3, 1), (40, 12), (120, 40)] {
                    let area = Rect::new(0, 0, w, h);
                    let (buf, full) = render_padded(&draw, area);
                    assert_no_outside_writes(&format!("{name}/{}", t.name), &buf, full, area);
                }
            }
        }
    }
}

#[test]
fn themed_widgets_fill_background() {
    let t = Theme::nord();
    let bg = t.background.expect("nord has a background");
    let b = render(Sparkline::new([1.0, 2.0, 3.0]).theme(t), 10, 2);
    assert!(b.content.iter().all(|c| c.bg == bg));
}

#[test]
fn base_style_beats_theme_background() {
    let b = render(
        Sparkline::new([1.0, 2.0])
            .theme(Theme::nord())
            .base_style(ratatui::style::Style::default().bg(Color::Rgb(9, 9, 9))),
        6,
        1,
    );
    assert!(b.content.iter().all(|c| c.bg == Color::Rgb(9, 9, 9)));
}

#[test]
fn explicit_color_wins_sparkline_and_gauge() {
    for t in Theme::all() {
        let red = Color::Rgb(250, 1, 2);
        let b = render(
            Sparkline::new([1.0, 5.0, 9.0]).theme(t.clone()).color(red),
            3,
            1,
        );
        assert!(
            b.content.iter().all(|c| c.fg == red),
            "sparkline {}",
            t.name
        );
        let b = render(
            Gauge::new(100.0)
                .range(0.0, 100.0)
                .theme(t.clone())
                .color(red),
            10,
            1,
        );
        assert!(b.content.iter().any(|c| c.fg == red), "gauge {}", t.name);
    }
}

#[test]
fn explicit_candle_colors_win() {
    let (up, down) = (Color::Rgb(1, 2, 3), Color::Rgb(4, 5, 6));
    for t in Theme::all() {
        let b = render(
            CandlestickChart::new(candles(Data::Normal))
                .theme(t.clone())
                .colors(up, down),
            60,
            20,
        );
        let cs = colors(&b);
        assert!(cs.contains(&up) && cs.contains(&down), "{}", t.name);
        for c in [t.bull, t.bear] {
            if c != up && c != down && ![t.foreground, t.axis, t.grid, t.muted].contains(&c) {
                assert!(!cs.contains(&c), "{}: theme {c:?} leaked", t.name);
            }
        }
    }
}

#[test]
fn explicit_waterfall_depth_renko_colors_win() {
    let (a, b2, c) = (
        Color::Rgb(11, 12, 13),
        Color::Rgb(21, 22, 23),
        Color::Rgb(31, 32, 33),
    );
    let t = Theme::dracula();
    let w = Waterfall::new()
        .initial(5.0)
        .delta("a", 3.0)
        .delta("b", -4.0)
        .total("t")
        .theme(t.clone())
        .colors(a, b2, c);
    let cs = colors(&render(w, 40, 12));
    assert!([a, b2, c].iter().all(|x| cs.contains(x)));

    let d = DepthChart::new([(9.0, 3.0), (8.0, 5.0)], [(11.0, 2.0), (12.0, 6.0)])
        .theme(t.clone())
        .bid_color(a)
        .ask_color(b2);
    let cs = colors(&render(d, 40, 12));
    assert!(cs.contains(&a) && cs.contains(&b2));

    let r = RenkoChart::new(&[1.0, 3.0, 6.0, 2.0, 0.0], 1.0)
        .theme(t)
        .up_color(a)
        .down_color(b2);
    let cs = colors(&render(r, 40, 12));
    assert!(cs.contains(&a) && cs.contains(&b2));
}

#[test]
fn theme_cycle_and_palette_lookup() {
    let all = Theme::all();
    let mut t = all[0].clone();
    for _ in 0..all.len() {
        t = t.next();
    }
    assert_eq!(t.name, all[0].name);
    assert_eq!(t.prev().next().name, t.name);
    let n = t.palette.len();
    assert_eq!(t.color(0), t.color(n), "palette wraps");
}

// ---- shared tui-nuances integration ----

use tui_nuances::{ThemeProvider, Themed};

fn bg_set(b: &Buffer) -> bool {
    b.content.iter().any(|c| c.bg != Color::Reset)
}

#[test]
fn theme_is_the_shared_nuances_type() {
    let t: tui_nuances::Theme = Theme::nord();
    assert_eq!(t.name, "nord");
}

#[test]
fn themed_trait_equals_builder_and_covers_builtins() {
    for t in Theme::all() {
        let a = render(Sparkline::new([1.0, 2.0, 3.0]).theme(&t), 8, 2);
        let b = render(Sparkline::new([1.0, 2.0, 3.0]).apply_theme(&t), 8, 2);
        assert_eq!(a, b, "{}", t.name);
        let g = render(Gauge::new(5.0).range(0.0, 10.0).apply_theme(&t), 10, 1);
        assert!(g.area.width == 10);
    }
}

#[test]
fn unthemed_has_no_background() {
    assert!(!bg_set(&render(Sparkline::new([1.0, 2.0, 3.0]), 8, 2)));
}

#[test]
fn explicit_axis_style_wins_in_any_order() {
    let red = Color::Rgb(250, 1, 2);
    let ax = || Axis::new().style(ratatui::style::Style::default().fg(red));
    let t = Theme::dracula();
    let before = render(
        LineChart::new(vec![Series::new("a").values([1.0, 3.0])])
            .x_axis(ax())
            .theme(&t),
        40,
        12,
    );
    let after = render(
        LineChart::new(vec![Series::new("a").values([1.0, 3.0])])
            .theme(&t)
            .x_axis(ax()),
        40,
        12,
    );
    assert!(colors(&before).contains(&red));
    assert!(colors(&after).contains(&red));
}

#[test]
fn axis_and_crosshair_are_themed() {
    let t = Theme::nord();
    let a = Axis::new().apply_theme(&t);
    let _ = Axis::new().theme(&t);
    let _ = a;
    let c = Crosshair::new(1.0, 2.0).apply_theme(&t);
    assert_eq!(c.style.fg, Some(t.crosshair));
    let red = ratatui::style::Style::default().fg(Color::Red);
    assert_eq!(Crosshair::new(1.0, 2.0).style(red).theme(&t).style, red);
}

#[test]
fn chart_ext_and_provider() {
    struct App;
    impl ThemeProvider for App {
        fn nuance_theme(&self) -> Theme {
            Theme::from_base(
                "app",
                None,
                Color::White,
                Color::Red,
                Color::Green,
                Color::Red,
            )
        }
    }
    let b = render(Sparkline::new([1.0, 2.0]).themed(&App), 4, 1);
    assert_eq!(b.area.width, 4);
    for t in Theme::all() {
        assert!(matches!(ChartThemeExt::gradient(&t), Gradient::Custom(_)));
    }
}
