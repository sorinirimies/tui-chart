//! Axis configuration shared by all charts.

use crate::util::{format_tick, nice_ticks, tick_step};
use ratatui::style::{Color, Style};

/// Configuration for a chart axis (title, bounds, ticks, grid, labels).
#[derive(Debug, Clone)]
pub struct Axis {
    pub(crate) title: Option<String>,
    pub(crate) bounds: Option<(f64, f64)>,
    pub(crate) ticks: usize,
    pub(crate) style: Style,
    pub(crate) label_style: Style,
    pub(crate) grid: bool,
    pub(crate) grid_style: Style,
    pub(crate) labels: Option<Vec<String>>,
    pub(crate) formatter: Option<fn(f64) -> String>,
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            title: None,
            bounds: None,
            ticks: 5,
            style: Style::default().fg(Color::DarkGray),
            label_style: Style::default().fg(Color::Gray),
            grid: false,
            grid_style: Style::default().fg(Color::DarkGray),
            labels: None,
            formatter: None,
        }
    }
}

impl Axis {
    /// Create an axis with defaults (auto bounds, 5 ticks, no grid).
    pub fn new() -> Self {
        Self::default()
    }

    /// Axis title (drawn at the top for y, at the bottom for x).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Fixed `(min, max)` bounds instead of auto-scaling.
    pub fn bounds(mut self, min: f64, max: f64) -> Self {
        self.bounds = Some((min, max));
        self
    }

    /// Desired number of ticks (min 2).
    pub fn ticks(mut self, ticks: usize) -> Self {
        self.ticks = ticks.max(2);
        self
    }

    /// Style of the axis line.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style of tick labels and the title.
    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    /// Draw dotted grid lines at each tick.
    pub fn grid(mut self, grid: bool) -> Self {
        self.grid = grid;
        self
    }

    /// Style of the grid lines.
    pub fn grid_style(mut self, style: Style) -> Self {
        self.grid_style = style;
        self
    }

    /// Custom, evenly spaced tick labels (overrides numeric ticks).
    pub fn labels<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.labels = Some(labels.into_iter().map(Into::into).collect());
        self
    }

    /// Custom number formatter for tick labels.
    pub fn formatter(mut self, f: fn(f64) -> String) -> Self {
        self.formatter = Some(f);
        self
    }

    /// `(fraction in 0..=1, label)` for each tick within `bounds`.
    pub(crate) fn tick_marks(&self, bounds: (f64, f64)) -> Vec<(f64, String)> {
        if let Some(labels) = &self.labels {
            let n = labels.len();
            return labels
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    let f = if n <= 1 {
                        0.0
                    } else {
                        i as f64 / (n - 1) as f64
                    };
                    (f, l.clone())
                })
                .collect();
        }
        let (lo, hi) = bounds;
        if hi <= lo {
            return vec![];
        }
        let ticks = nice_ticks(lo, hi, self.ticks);
        let step = tick_step(&ticks);
        ticks
            .iter()
            .map(|&v| {
                let label = match self.formatter {
                    Some(f) => f(v),
                    None => format_tick(v, step),
                };
                ((v - lo) / (hi - lo), label)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_marks() {
        let m = Axis::new().tick_marks((0.0, 100.0));
        assert_eq!(m.first().unwrap().1, "0");
        assert_eq!(m.last().unwrap().1, "100");
        assert!((m.last().unwrap().0 - 1.0).abs() < 1e-12);
    }

    #[test]
    fn custom_labels_even() {
        let m = Axis::new().labels(["a", "b", "c"]).tick_marks((0.0, 1.0));
        assert_eq!(m.len(), 3);
        assert_eq!(m[1], (0.5, "b".to_string()));
    }

    #[test]
    fn formatter_applies() {
        let m = Axis::new()
            .formatter(|v| format!("${v}"))
            .tick_marks((0.0, 10.0));
        assert!(m[0].1.starts_with('$'));
    }

    #[test]
    fn builder_sets_fields() {
        let a = Axis::new().title("t").bounds(1.0, 2.0).ticks(0).grid(true);
        assert_eq!(a.title.as_deref(), Some("t"));
        assert_eq!(a.bounds, Some((1.0, 2.0)));
        assert_eq!(a.ticks, 2);
        assert!(a.grid);
    }

    #[test]
    fn degenerate_bounds_no_marks() {
        assert!(Axis::new().tick_marks((1.0, 1.0)).is_empty());
    }
}
