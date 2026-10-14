//! Axis configuration shared by all charts.

use crate::theme::Theme;
use crate::util::{format_tick, nice_ticks, tick_step};
use ratatui::style::{Color, Style};
use std::fmt;
use std::sync::Arc;

/// Shared tick-label formatter.
pub(crate) type Formatter = Arc<dyn Fn(f64) -> String + Send + Sync>;

/// Look of grid lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GridStyle {
    /// Dotted (`·`), the default.
    #[default]
    Dotted,
    /// Dashed (`╌` / `╎`).
    Dashed,
    /// Solid thin lines (`─` / `│`).
    Solid,
}

impl GridStyle {
    /// Glyph for a horizontal (`true`) or vertical grid line.
    pub(crate) fn glyph(self, horizontal: bool) -> char {
        match (self, horizontal) {
            (GridStyle::Dotted, _) => '·',
            (GridStyle::Dashed, true) => '╌',
            (GridStyle::Dashed, false) => '╎',
            (GridStyle::Solid, true) => '─',
            (GridStyle::Solid, false) => '│',
        }
    }
}

/// Configuration for a chart axis (title, bounds, ticks, grid, labels).
#[derive(Clone)]
pub struct Axis {
    pub(crate) title: Option<String>,
    pub(crate) bounds: Option<(f64, f64)>,
    pub(crate) ticks: usize,
    pub(crate) style: Style,
    pub(crate) label_style: Style,
    pub(crate) grid: bool,
    pub(crate) grid_style: Style,
    pub(crate) labels: Option<Vec<String>>,
    pub(crate) formatter: Option<Formatter>,
    pub(crate) grid_kind: GridStyle,
    pub(crate) explicit: u8,
}

const EXPL_STYLE: u8 = 1;
const EXPL_LABEL: u8 = 2;
const EXPL_GRID: u8 = 4;

impl fmt::Debug for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Axis")
            .field("title", &self.title)
            .field("bounds", &self.bounds)
            .field("ticks", &self.ticks)
            .field("grid", &self.grid)
            .field("grid_kind", &self.grid_kind)
            .field("labels", &self.labels)
            .field("formatter", &self.formatter.as_ref().map(|_| "<fn>"))
            .finish()
    }
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
            grid_kind: GridStyle::default(),
            explicit: 0,
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
        self.explicit |= EXPL_STYLE;
        self
    }

    /// Style of tick labels and the title.
    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self.explicit |= EXPL_LABEL;
        self
    }

    setter!(
        /// Draw dotted grid lines at each tick.
        grid: bool
    );

    /// Style of the grid lines.
    pub fn grid_style(mut self, style: Style) -> Self {
        self.grid_style = style;
        self.explicit |= EXPL_GRID;
        self
    }

    setter!(
        /// Grid line look: dotted (default), dashed or solid.
        grid_lines => grid_kind: GridStyle
    );

    /// Apply a theme to every style not set explicitly on this axis.
    pub(crate) fn theme_in_place(&mut self, t: &Theme) {
        if self.explicit & EXPL_STYLE == 0 {
            self.style = t.axis_style();
        }
        if self.explicit & EXPL_LABEL == 0 {
            self.label_style = t.label_style();
        }
        if self.explicit & EXPL_GRID == 0 {
            self.grid_style = t.grid_style();
        }
    }

    /// Style axis line, tick labels and grid from `theme`; styles set
    /// explicitly on this axis win regardless of call order.
    #[must_use]
    #[cfg(feature = "theme")]
    pub fn theme(mut self, theme: impl Into<Theme>) -> Self {
        self.theme_in_place(&theme.into());
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

    /// Custom number formatter for tick labels. Accepts function pointers and
    /// (capturing) closures.
    pub fn formatter(mut self, f: impl Fn(f64) -> String + Send + Sync + 'static) -> Self {
        self.formatter = Some(Arc::new(f));
        self
    }

    /// Alias of [`Axis::formatter`] taking an already shared formatter.
    pub fn formatter_fn(mut self, f: Arc<dyn Fn(f64) -> String + Send + Sync>) -> Self {
        self.formatter = Some(f);
        self
    }

    /// Format a value with the custom formatter, if any.
    pub(crate) fn format(&self, v: f64) -> Option<String> {
        self.formatter.as_ref().map(|f| f(v))
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
                let label = match self.format(v) {
                    Some(l) => l,
                    None => format_tick(v, step),
                };
                ((v - lo) / (hi - lo), label)
            })
            .collect()
    }
}

#[cfg(feature = "theme")]
impl tui_nuances::Themed for Axis {
    fn apply_theme(self, theme: &Theme) -> Self {
        self.theme(theme)
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
    fn closure_formatter_captures() {
        let unit = String::from("kg");
        let m = Axis::new()
            .formatter(move |v| format!("{v}{unit}"))
            .tick_marks((0.0, 10.0));
        assert!(m[0].1.ends_with("kg"));
    }

    #[cfg(feature = "theme")]
    #[test]
    fn theme_respects_explicit() {
        let t = Theme::dracula();
        let mut a = Axis::new().style(Style::default().fg(Color::Red));
        a.theme_in_place(&t);
        assert_eq!(a.style.fg, Some(Color::Red));
        assert_eq!(a.label_style.fg, Some(t.muted));
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
