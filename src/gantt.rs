//! Gantt: timeline of tasks with progress fill and optional "today" marker.

use crate::theme::{self, Theme};
use crate::util::{format_tick, nice_ticks};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

/// One Gantt task.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Task {
    /// Row label.
    pub label: String,
    /// Start time.
    pub start: f64,
    /// End time.
    pub end: f64,
    /// Completion in `0.0..=1.0`.
    pub progress: f64,
    /// Optional bar color (overrides the palette).
    pub color: Option<Color>,
}

impl Task {
    /// New task from `start` to `end` with no progress.
    pub fn new(label: impl Into<String>, start: f64, end: f64) -> Self {
        Self {
            label: label.into(),
            start,
            end,
            progress: 0.0,
            color: None,
        }
    }

    /// Completion fraction (clamped to 0..=1).
    pub fn progress(mut self, p: f64) -> Self {
        self.progress = if p.is_finite() {
            p.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self
    }

    opt_setter!(
        /// Bar color.
        color: Color
    );
}

/// Gantt chart widget.
#[derive(Debug, Clone)]
pub struct Gantt {
    tasks: Vec<Task>,
    palette: Option<Vec<Color>>,
    theme: Option<Theme>,
    base: Option<Style>,
    today: Option<f64>,
    range: Option<(f64, f64)>,
    block: Option<Block<'static>>,
}

impl Gantt {
    /// Create from tasks.
    pub fn new(tasks: impl IntoIterator<Item = Task>) -> Self {
        Self {
            tasks: tasks.into_iter().collect(),
            palette: None,
            theme: None,
            base: None,
            today: None,
            range: None,
            block: None,
        }
    }

    opt_setter!(
        /// Mark the current time with a vertical line.
        today: f64
    );

    /// Fixed time range (default: span of all tasks).
    pub fn range(mut self, start: f64, end: f64) -> Self {
        self.range = Some((start, end));
        self
    }

    /// Palette cycled over tasks without an explicit color.
    pub fn palette(mut self, p: Vec<Color>) -> Self {
        if !p.is_empty() {
            self.palette = Some(p);
        }
        self
    }

    theme_setter!(
        /// Apply a [`Theme`]: palette task colors, bear-colored today marker,
        /// axis colors and background fill. Explicit task colors and
        /// [`Gantt::palette`] win.
        theme
    );

    base_style_setter!(base);

    fn task_color(&self, i: usize) -> Color {
        if let Some(p) = &self.palette {
            return p[i % p.len()];
        }
        match &self.theme {
            Some(t) => t.color(i),
            None => [
                Color::Cyan,
                Color::Green,
                Color::Yellow,
                Color::Magenta,
                Color::LightBlue,
            ][i % 5],
        }
    }

    block_setter!(block, 'static);

    fn time_range(&self) -> (f64, f64) {
        if let Some((a, b)) = self.range {
            if a.is_finite() && b.is_finite() && b > a {
                return (a, b);
            }
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for t in self
            .tasks
            .iter()
            .filter(|t| t.start.is_finite() && t.end.is_finite())
        {
            lo = lo.min(t.start.min(t.end));
            hi = hi.max(t.start.max(t.end));
        }
        if !lo.is_finite() {
            (0.0, 1.0)
        } else if hi <= lo {
            (lo, lo + 1.0)
        } else {
            (lo, hi)
        }
    }
}

fn put_str(buf: &mut Buffer, x: u16, y: u16, max_x: u16, s: &str, style: Style) {
    for (i, ch) in s.chars().enumerate() {
        let cx = x as u32 + i as u32;
        if cx >= max_x as u32 {
            break;
        }
        buf[(cx as u16, y)].set_char(ch).set_style(style);
    }
}

impl Gantt {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        if inner.width < 4 || inner.height < 2 || self.tasks.is_empty() {
            return;
        }
        let gw = self
            .tasks
            .iter()
            .map(|t| t.label.chars().count())
            .max()
            .unwrap_or(0) as u16;
        let gutter = (gw + 1).min(inner.width / 3);
        let plot = Rect::new(
            inner.x + gutter,
            inner.y,
            inner.width - gutter,
            inner.height - 1,
        );
        if plot.width == 0 || plot.height == 0 {
            return;
        }
        let (lo, hi) = self.time_range();
        let col = |t: f64| -> u16 {
            let f = ((t - lo) / (hi - lo)).clamp(0.0, 1.0);
            plot.x + (f * (plot.width - 1) as f64).round() as u16
        };
        let dim = Style::default().fg(self.theme.as_ref().map_or(Color::DarkGray, |t| t.grid));
        let text = self
            .theme
            .as_ref()
            .map_or(Style::default(), |t| t.label_style());

        // Time axis
        let ticks = nice_ticks(lo, hi, (plot.width as usize / 10).max(2));
        let step = if ticks.len() >= 2 {
            ticks[1] - ticks[0]
        } else {
            1.0
        };
        let ay = inner.bottom() - 1;
        for v in &ticks {
            let x = col(*v);
            for y in plot.y..plot.bottom() {
                buf[(x, y)].set_char('┊').set_style(dim);
            }
            put_str(buf, x, ay, inner.right(), &format_tick(*v, step), dim);
        }

        for (i, t) in self.tasks.iter().enumerate().take(plot.height as usize) {
            let y = plot.y + i as u16;
            let l: String = t
                .label
                .chars()
                .take(gutter.saturating_sub(1) as usize)
                .collect();
            put_str(buf, inner.x, y, plot.x, &l, text);
            if !t.start.is_finite() || !t.end.is_finite() {
                continue;
            }
            let color = t.color.unwrap_or_else(|| self.task_color(i));
            let (a, b) = (col(t.start.min(t.end)), col(t.start.max(t.end)));
            let len = (b - a + 1) as f64;
            let done = (t.progress.clamp(0.0, 1.0) * len).round() as u16;
            for (k, x) in (a..=b).enumerate() {
                let (ch, st) = if (k as u16) < done {
                    ('█', Style::default().fg(color))
                } else {
                    ('░', Style::default().fg(color))
                };
                buf[(x, y)].set_char(ch).set_style(st);
            }
        }

        if let Some(t) = self.today.filter(|t| t.is_finite() && *t >= lo && *t <= hi) {
            let x = col(t);
            let st = Style::default().fg(self.theme.as_ref().map_or(Color::Red, |t| t.bear));
            for y in plot.y..plot.bottom() {
                buf[(x, y)].set_char('┃').set_style(st);
            }
            put_str(buf, x, ay, inner.right(), "▲", st);
        }
    }
}

widget_impls!(Gantt);

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

    fn render(g: &Gantt, w: u16, h: u16) -> Buffer {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        g.render(area, &mut buf);
        buf
    }

    fn count(b: &Buffer, s: &str) -> usize {
        b.content().iter().filter(|c| c.symbol() == s).count()
    }

    #[test]
    fn progress_fill() {
        let g = Gantt::new([Task::new("a", 0.0, 10.0).progress(0.5)]);
        let b = render(&g, 30, 4);
        assert!(count(&b, "█") > 0 && count(&b, "░") > 0);
        let full = render(
            &Gantt::new([Task::new("a", 0.0, 10.0).progress(1.0)]),
            30,
            4,
        );
        assert_eq!(count(&full, "░"), 0);
    }

    #[test]
    fn today_marker_and_labels() {
        let g =
            Gantt::new([Task::new("design", 0.0, 5.0), Task::new("build", 5.0, 10.0)]).today(5.0);
        let b = render(&g, 40, 5);
        assert!(count(&b, "┃") >= 2);
        let row0: String = (0..40).map(|x| b[(x, 0)].symbol().to_string()).collect();
        assert!(row0.starts_with("design"));
    }

    #[test]
    fn theme_colors_and_override() {
        let t = Theme::gruvbox();
        let g = Gantt::new([
            Task::new("a", 0.0, 5.0),
            Task::new("b", 5.0, 10.0).color(Color::Red),
        ])
        .today(5.0)
        .theme(&t);
        let b = render(&g, 40, 5);
        let fg = |c: Color| b.content().iter().any(|x| x.symbol() == "░" && x.fg == c);
        assert!(fg(t.color(0)));
        assert!(fg(Color::Red));
        assert!(!fg(t.color(1)));
        assert!(b
            .content()
            .iter()
            .any(|x| x.symbol() == "┃" && x.fg == t.bear));
        assert_eq!(b[(39, 0)].bg, t.background.unwrap());
    }

    #[test]
    fn tiny_and_nan_no_panic() {
        let g = Gantt::new([
            Task::new("x", f64::NAN, 1.0),
            Task::new("y", 3.0, 3.0).progress(f64::NAN),
        ])
        .today(3.0)
        .block(Block::bordered());
        for w in 0..8 {
            for h in 0..6 {
                render(&g, w, h);
            }
        }
        render(&Gantt::new([]), 10, 5);
    }
}
