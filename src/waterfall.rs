//! Waterfall: delta steps with a running total.

use crate::util::{format_compact, format_tick, nice_ticks};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

#[derive(Debug, Clone)]
enum Kind {
    Delta(f64),
    Total,
}

#[derive(Debug, Clone)]
struct Step {
    label: String,
    kind: Kind,
}

/// Waterfall chart widget.
#[derive(Debug, Clone)]
pub struct Waterfall {
    steps: Vec<Step>,
    initial: f64,
    up: Color,
    down: Color,
    total: Color,
    connectors: bool,
    show_values: bool,
    block: Option<Block<'static>>,
}

impl Default for Waterfall {
    fn default() -> Self {
        Self::new()
    }
}

impl Waterfall {
    /// Empty chart starting at 0.
    pub fn new() -> Self {
        Self {
            steps: vec![],
            initial: 0.0,
            up: Color::Green,
            down: Color::Red,
            total: Color::Blue,
            connectors: true,
            show_values: true,
            block: None,
        }
    }

    /// Add a delta step (non-finite deltas count as 0).
    pub fn delta(mut self, label: impl Into<String>, v: f64) -> Self {
        let v = if v.is_finite() { v } else { 0.0 };
        self.steps.push(Step {
            label: label.into(),
            kind: Kind::Delta(v),
        });
        self
    }

    /// Add a total bar showing the running total so far.
    pub fn total(mut self, label: impl Into<String>) -> Self {
        self.steps.push(Step {
            label: label.into(),
            kind: Kind::Total,
        });
        self
    }

    /// Starting value of the running total (default 0).
    pub fn initial(mut self, v: f64) -> Self {
        self.initial = if v.is_finite() { v } else { 0.0 };
        self
    }

    /// Colors for increases, decreases and totals.
    pub fn colors(mut self, up: Color, down: Color, total: Color) -> Self {
        self.up = up;
        self.down = down;
        self.total = total;
        self
    }

    /// Draw connector lines between bars (default on).
    pub fn connectors(mut self, on: bool) -> Self {
        self.connectors = on;
        self
    }

    /// Print values above/below bars (default on).
    pub fn show_values(mut self, on: bool) -> Self {
        self.show_values = on;
        self
    }

    /// Wrap in a block.
    pub fn block(mut self, b: Block<'static>) -> Self {
        self.block = Some(b);
        self
    }

    /// Bars as `(start, end, is_total)` with the running total applied.
    pub fn levels(&self) -> Vec<(f64, f64, bool)> {
        let mut run = self.initial;
        self.steps
            .iter()
            .map(|s| match s.kind {
                Kind::Delta(d) => {
                    let start = run;
                    run += d;
                    (start, run, false)
                }
                Kind::Total => (0.0, run, true),
            })
            .collect()
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

impl Widget for &Waterfall {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        let inner = match &self.block {
            Some(b) => {
                let i = b.inner(area);
                b.clone().render(area, buf);
                i
            }
            None => area,
        };
        if inner.width < 4 || inner.height < 4 || self.steps.is_empty() {
            return;
        }
        let levels = self.levels();
        let mut lo = 0f64.min(self.initial);
        let mut hi = 0f64.max(self.initial);
        for (s, e, _) in &levels {
            lo = lo.min(*s).min(*e);
            hi = hi.max(*s).max(*e);
        }
        if !(hi - lo).is_finite() || hi <= lo {
            hi = lo + 1.0;
        }
        let dim = Style::default().fg(Color::DarkGray);
        let ticks = nice_ticks(lo, hi, (inner.height as usize / 3).max(2));
        let tstep = if ticks.len() >= 2 {
            ticks[1] - ticks[0]
        } else {
            1.0
        };
        let labels: Vec<String> = ticks.iter().map(|v| format_tick(*v, tstep)).collect();
        let lw = labels.iter().map(|l| l.chars().count()).max().unwrap_or(1) as u16;
        let gutter = (lw + 1).min(inner.width / 2);
        // Top row reserved for value labels, bottom row for step labels.
        let plot = Rect::new(
            inner.x + gutter,
            inner.y + 1,
            inner.width - gutter,
            inner.height - 2,
        );
        if plot.width == 0 || plot.height == 0 {
            return;
        }
        let row = |v: f64| -> u16 {
            let f = ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
            plot.bottom() - 1 - (f * (plot.height - 1) as f64).round() as u16
        };
        for (v, l) in ticks.iter().zip(&labels) {
            let y = row(*v);
            let w = l.chars().count() as u16;
            put_str(
                buf,
                inner.x + gutter.saturating_sub(w + 1),
                y,
                plot.x,
                l,
                dim,
            );
        }
        // zero line
        if lo < 0.0 && hi > 0.0 {
            let y = row(0.0);
            for x in plot.x..plot.right() {
                buf[(x, y)].set_char('─').set_style(dim);
            }
        }

        let n = self.steps.len();
        let slot = plot.width as f64 / n as f64;
        let mut prev_end: Option<(f64, u16)> = None; // (level, right edge x)
        for (i, ((s, e, is_total), step)) in levels.iter().zip(&self.steps).enumerate() {
            let x0 = plot.x + (i as f64 * slot).floor() as u16;
            let x1 = plot.x + (((i + 1) as f64 * slot).floor() as u16).max(x0 - plot.x + 1);
            let x1 = x1.min(plot.right());
            if x0 >= x1 {
                continue;
            }
            let slot_w = x1 - x0;
            let bw = if slot_w >= 3 { slot_w - 1 } else { slot_w };
            let color = if *is_total {
                self.total
            } else if e >= s {
                self.up
            } else {
                self.down
            };
            let st = Style::default().fg(color);
            let (ra, rb) = (row(*s), row(*e));
            let (top, bot) = (ra.min(rb), ra.max(rb));
            if self.connectors {
                if let Some((lvl, px)) = prev_end {
                    let y = row(lvl);
                    for x in px..x0 {
                        buf[(x, y)].set_char('┄').set_style(dim);
                    }
                }
            }
            for y in top..=bot {
                for x in x0..x0 + bw {
                    buf[(x, y)].set_char('█').set_style(st);
                }
            }
            prev_end = Some((*e, x0 + bw));
            if self.show_values {
                let v = if *is_total {
                    format_compact(*e)
                } else if e >= s {
                    format!("+{}", format_compact(e - s))
                } else {
                    format_compact(e - s)
                };
                let vw = v.chars().count() as u16;
                let off = bw.saturating_sub(vw) / 2;
                let ty = if top > plot.y {
                    Some(top - 1)
                } else if inner.y < plot.y {
                    Some(inner.y)
                } else {
                    None
                };
                if let Some(y) = ty {
                    put_str(buf, x0 + off, y, x1, &v, st);
                }
            }
            let l: String = step.label.chars().take(slot_w as usize).collect();
            put_str(
                buf,
                x0,
                inner.bottom() - 1,
                inner.right(),
                &l,
                Style::default(),
            );
        }
    }
}

impl Widget for Waterfall {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(w: &Waterfall, width: u16, height: u16) -> Buffer {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        w.render(area, &mut buf);
        buf
    }

    fn sample() -> Waterfall {
        Waterfall::new()
            .delta("Sales", 100.0)
            .delta("Cost", -40.0)
            .total("Net")
    }

    #[test]
    fn running_total() {
        let l = sample().levels();
        assert_eq!(
            l,
            vec![(0.0, 100.0, false), (100.0, 60.0, false), (0.0, 60.0, true)]
        );
    }

    #[test]
    fn colors_by_sign() {
        let b = render(&sample(), 40, 14);
        let fg = |c: Color| b.content().iter().any(|x| x.symbol() == "█" && x.fg == c);
        assert!(fg(Color::Green) && fg(Color::Red) && fg(Color::Blue));
        assert!(b.content().iter().any(|x| x.symbol() == "┄"));
    }

    #[test]
    fn tiny_and_empty_no_panic() {
        let w = sample().delta("nan", f64::NAN).block(Block::bordered());
        for x in 0..8 {
            for y in 0..6 {
                render(&w, x, y);
            }
        }
        render(&Waterfall::new(), 10, 5);
    }
}
