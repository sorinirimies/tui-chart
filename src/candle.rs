//! Candlestick / OHLC chart for price data (stocks, crypto, FX…).

use crate::axis::Axis;
use crate::layout::{self, text_width};
use crate::util::{format_compact, nice_bounds};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

/// One OHLC(V) bar.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Candle {
    /// Opening price.
    pub open: f64,
    /// Highest price.
    pub high: f64,
    /// Lowest price.
    pub low: f64,
    /// Closing price.
    pub close: f64,
    /// Traded volume.
    pub volume: f64,
    /// Optional x-axis label (e.g. time).
    pub label: Option<String>,
}

impl Candle {
    /// Create a candle; `high`/`low` are widened to include open and close.
    pub fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            open,
            high: high.max(open).max(close),
            low: low.min(open).min(close),
            close,
            volume: 0.0,
            label: None,
        }
    }

    /// Attach volume.
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// Attach an x-axis label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Close ≥ open.
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }

    fn is_valid(&self) -> bool {
        [self.open, self.high, self.low, self.close]
            .iter()
            .all(|v| v.is_finite())
    }
}

/// Visual style of each bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CandleStyle {
    /// Japanese candlesticks with body and wicks.
    #[default]
    Candle,
    /// OHLC bars: vertical range with open (left) / close (right) ticks.
    Ohlc,
    /// Heikin-Ashi smoothed candles.
    HeikinAshi,
}

/// Technical overlay (e.g. moving average) aligned to the candle index.
#[derive(Debug, Clone)]
pub struct Overlay {
    /// Legend name.
    pub name: String,
    /// One value per candle (`None` = no value).
    pub values: Vec<Option<f64>>,
    /// Dot color.
    pub color: Color,
}

impl Overlay {
    /// Create an overlay.
    pub fn new(name: impl Into<String>, values: Vec<Option<f64>>, color: Color) -> Self {
        Self {
            name: name.into(),
            values,
            color,
        }
    }
}

/// Simple moving average of closes (`None` until `period` values exist).
pub fn sma(candles: &[Candle], period: usize) -> Vec<Option<f64>> {
    let period = period.max(1);
    (0..candles.len())
        .map(|i| {
            if i + 1 < period {
                None
            } else {
                let s: f64 = candles[i + 1 - period..=i].iter().map(|c| c.close).sum();
                Some(s / period as f64)
            }
        })
        .collect()
}

/// Exponential moving average of closes.
pub fn ema(candles: &[Candle], period: usize) -> Vec<Option<f64>> {
    let period = period.max(1);
    let k = 2.0 / (period as f64 + 1.0);
    let mut prev: Option<f64> = None;
    candles
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if i + 1 < period {
                return None;
            }
            let v = match prev {
                None => {
                    candles[i + 1 - period..=i]
                        .iter()
                        .map(|c| c.close)
                        .sum::<f64>()
                        / period as f64
                }
                Some(p) => c.close * k + p * (1.0 - k),
            };
            prev = Some(v);
            Some(v)
        })
        .collect()
}

/// Convert candles to Heikin-Ashi candles.
pub fn heikin_ashi(candles: &[Candle]) -> Vec<Candle> {
    let mut out: Vec<Candle> = Vec::with_capacity(candles.len());
    for (i, c) in candles.iter().enumerate() {
        let close = (c.open + c.high + c.low + c.close) / 4.0;
        let open = if i == 0 {
            (c.open + c.close) / 2.0
        } else {
            (out[i - 1].open + out[i - 1].close) / 2.0
        };
        out.push(Candle {
            open,
            high: c.high.max(open).max(close),
            low: c.low.min(open).min(close),
            close,
            volume: c.volume,
            label: c.label.clone(),
        });
    }
    out
}

const EIGHTHS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Candlestick / OHLC chart with optional volume panel and overlays.
#[derive(Debug, Clone)]
pub struct CandlestickChart {
    candles: Vec<Candle>,
    style: CandleStyle,
    up_color: Color,
    down_color: Color,
    candle_width: u16,
    gap: u16,
    show_volume: bool,
    last_price_line: bool,
    scroll: usize,
    overlays: Vec<Overlay>,
    block: Option<Block<'static>>,
    x_axis: Axis,
    y_axis: Axis,
}

impl CandlestickChart {
    /// Create a chart from candles.
    pub fn new(candles: Vec<Candle>) -> Self {
        Self {
            candles,
            style: CandleStyle::Candle,
            up_color: Color::Green,
            down_color: Color::Red,
            candle_width: 1,
            gap: 1,
            show_volume: false,
            last_price_line: true,
            scroll: 0,
            overlays: vec![],
            block: None,
            x_axis: Axis::new().ticks(6),
            y_axis: Axis::new().grid(true),
        }
    }

    /// Bar style.
    pub fn style(mut self, style: CandleStyle) -> Self {
        self.style = style;
        self
    }

    /// Colors for bullish / bearish bars.
    pub fn colors(mut self, up: Color, down: Color) -> Self {
        self.up_color = up;
        self.down_color = down;
        self
    }

    /// Body width in cells (ignored for OHLC bars; min 1).
    pub fn candle_width(mut self, w: u16) -> Self {
        self.candle_width = w.max(1);
        self
    }

    /// Empty cells between bars.
    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    /// Show a volume histogram under the price panel.
    pub fn show_volume(mut self, on: bool) -> Self {
        self.show_volume = on;
        self
    }

    /// Dashed horizontal line at the last visible close.
    pub fn last_price_line(mut self, on: bool) -> Self {
        self.last_price_line = on;
        self
    }

    /// Hide the newest `n` candles (scroll back in time).
    pub fn scroll(mut self, n: usize) -> Self {
        self.scroll = n;
        self
    }

    /// Add an indicator overlay.
    pub fn overlay(mut self, overlay: Overlay) -> Self {
        self.overlays.push(overlay);
        self
    }

    /// Wrap in a block.
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// X axis (title / label style; labels come from [`Candle::label`]).
    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = axis;
        self
    }

    /// Y (price) axis.
    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = axis;
        self
    }

    fn bar_width(&self) -> u16 {
        match self.style {
            CandleStyle::Ohlc => 1,
            _ => self.candle_width,
        }
    }

    /// Index range `start..end` of candles visible in `slots` slots.
    pub(crate) fn visible_range(&self, slots: usize) -> (usize, usize) {
        let end = self.candles.len().saturating_sub(self.scroll);
        (end.saturating_sub(slots), end)
    }

    fn color_of(&self, c: &Candle) -> Color {
        if c.is_bullish() {
            self.up_color
        } else {
            self.down_color
        }
    }

    fn render_inner(&self, area: Rect, buf: &mut Buffer) {
        let inner = layout::render_block(&self.block, area, buf);
        let src: Vec<Candle> = match self.style {
            CandleStyle::HeikinAshi => heikin_ashi(&self.candles),
            _ => self.candles.clone(),
        };
        let bw = self.bar_width();
        let slot = (bw + self.gap) as usize;
        // provisional width to pick the visible window (label width impacts it slightly)
        let (start, end) = self.visible_range((inner.width as usize).max(1) / slot);
        let vis: Vec<&Candle> = src[start.min(src.len())..end.min(src.len())]
            .iter()
            .collect();
        let valid: Vec<&&Candle> = vis.iter().filter(|c| c.is_valid()).collect();

        let (lo, hi) = self.y_axis.bounds.unwrap_or_else(|| {
            if valid.is_empty() {
                (0.0, 1.0)
            } else {
                let lo = valid.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
                let hi = valid
                    .iter()
                    .map(|c| c.high)
                    .fold(f64::NEG_INFINITY, f64::max);
                nice_bounds(lo, hi, self.y_axis.ticks)
            }
        });
        if hi <= lo {
            return;
        }
        let y_marks = self.y_axis.tick_marks((lo, hi));
        let yw = y_marks
            .iter()
            .map(|(_, l)| text_width(l))
            .max()
            .unwrap_or(0);
        let has_labels = vis.iter().any(|c| c.label.is_some());
        let Some(a) = layout::compute(
            inner,
            yw,
            has_labels,
            self.x_axis.title.is_some(),
            self.y_axis.title.is_some(),
        ) else {
            return;
        };
        let plot = a.plot;
        // recompute window with the real plot width
        let (start, end) =
            self.visible_range((plot.width as usize + self.gap as usize) / slot.max(1));
        let vis: Vec<&Candle> = src[start.min(src.len())..end.min(src.len())]
            .iter()
            .collect();

        let vol_h = if self.show_volume && plot.height >= 8 {
            (plot.height / 4).max(2)
        } else {
            0
        };
        let price = Rect::new(plot.x, plot.y, plot.width, plot.height - vol_h);
        let sub_n = price.height as f64 * 2.0;
        let sub = |p: f64| -> i32 {
            ((hi - p) / (hi - lo) * (sub_n - 1.0))
                .round()
                .clamp(0.0, sub_n - 1.0) as i32
        };

        // grid + y labels
        let y_ticks: Vec<(u16, String)> = y_marks
            .iter()
            .map(|(f, l)| (layout::frac_row(*f, price.height, 2), l.clone()))
            .collect();
        if self.y_axis.grid {
            let rows: Vec<u16> = y_ticks.iter().map(|t| t.0).collect();
            layout::draw_grid(buf, price, &[], &rows, self.y_axis.grid_style);
        }

        // last price line
        if self.last_price_line {
            if let Some(c) = vis.iter().rev().find(|c| c.is_valid()) {
                let row = (sub(c.close) / 2) as u16;
                let style = Style::default().fg(self.color_of(c));
                for x in price.x..price.right() {
                    if let Some(cell) = buf.cell_mut((x, price.y + row)) {
                        if cell.symbol() == " " || cell.symbol() == "·" {
                            cell.set_char('┈').set_style(style);
                        }
                    }
                }
            }
        }

        let max_vol = vis
            .iter()
            .map(|c| c.volume)
            .filter(|v| v.is_finite())
            .fold(0.0f64, f64::max);
        let mut x_ticks: Vec<(u16, String)> = vec![];
        for (i, c) in vis.iter().enumerate() {
            let x0 = price.x + (i * slot) as u16;
            if x0 + bw > price.right() {
                break;
            }
            let centre = x0 + bw / 2;
            if let Some(l) = &c.label {
                x_ticks.push((centre - plot.x, l.clone()));
            }
            if !c.is_valid() {
                continue;
            }
            let color = self.color_of(c);
            let st = Style::default().fg(color);
            if self.style == CandleStyle::Ohlc {
                let (r_h, r_l) = ((sub(c.high) / 2) as u16, (sub(c.low) / 2) as u16);
                let (r_o, r_c) = ((sub(c.open) / 2) as u16, (sub(c.close) / 2) as u16);
                for r in r_h..=r_l {
                    let ch = match (r == r_o, r == r_c) {
                        (true, true) => '┼',
                        (true, false) => '┤',
                        (false, true) => '├',
                        _ => '│',
                    };
                    buf.set_string(centre, price.y + r, ch.to_string(), st);
                }
            } else {
                let (b0, b1) = {
                    let (a_, b_) = (sub(c.open), sub(c.close));
                    (a_.min(b_), a_.max(b_))
                };
                let (w0, w1) = (sub(c.high), sub(c.low));
                for r in (w0 / 2)..=(w1 / 2) {
                    let (t, b) = (2 * r, 2 * r + 1);
                    let body_t = t >= b0 && t <= b1;
                    let body_b = b >= b0 && b <= b1;
                    let wick_t = t >= w0 && t <= w1;
                    let wick_b = b >= w0 && b <= w1;
                    let body_ch = match (body_t, body_b) {
                        (true, true) => Some('█'),
                        (true, false) => Some('▀'),
                        (false, true) => Some('▄'),
                        _ => None,
                    };
                    let wick_ch = match (wick_t, wick_b) {
                        (true, true) => Some('│'),
                        (true, false) => Some('╵'),
                        (false, true) => Some('╷'),
                        _ => None,
                    };
                    for dx in 0..bw {
                        let ch = if dx == bw / 2 {
                            body_ch.or(wick_ch)
                        } else {
                            body_ch
                        };
                        if let Some(ch) = ch {
                            buf.set_string(x0 + dx, price.y + r as u16, ch.to_string(), st);
                        }
                    }
                }
                // doji: make sure body is visible
                if b0 == b1 && bw > 1 {
                    // body already drawn with half block above
                }
            }
            // volume
            if vol_h > 0 && max_vol > 0.0 && c.volume.is_finite() && c.volume > 0.0 {
                let total = (c.volume / max_vol * vol_h as f64 * 8.0).round().max(1.0) as i32;
                for r in 0..vol_h as i32 {
                    let fill = (total - r * 8).clamp(0, 8) as usize;
                    if fill == 0 {
                        break;
                    }
                    let y = plot.bottom() - 1 - r as u16;
                    for dx in 0..bw {
                        buf.set_string(x0 + dx, y, EIGHTHS[fill].to_string(), st);
                    }
                }
            }
        }

        // overlays on top of bars
        for ov in &self.overlays {
            for (i, _) in vis.iter().enumerate() {
                let Some(Some(v)) = ov.values.get(start + i) else {
                    continue;
                };
                if !v.is_finite() || *v < lo || *v > hi {
                    continue;
                }
                let x = price.x + (i * slot) as u16 + bw / 2;
                if x >= price.right() {
                    continue;
                }
                let row = (sub(*v) / 2) as u16;
                buf.set_string(x, price.y + row, "●", Style::default().fg(ov.color));
            }
        }

        layout::draw_axes(buf, &a, self.x_axis.style, self.y_axis.style);
        layout::draw_y_labels(
            buf,
            Rect::new(a.y_labels.x, price.y, a.y_labels.width, price.height),
            &y_ticks,
            self.y_axis.label_style,
        );
        if vol_h > 0 {
            let top = Rect::new(a.y_labels.x, plot.bottom() - vol_h, a.y_labels.width, vol_h);
            let label = format_compact(max_vol);
            layout::draw_y_labels(
                buf,
                top,
                &[(0, label)],
                Style::default().fg(Color::DarkGray),
            );
        }
        layout::draw_x_labels(buf, a.x_labels, &x_ticks, self.x_axis.label_style);
        layout::draw_titles(
            buf,
            inner,
            &a,
            self.x_axis
                .title
                .as_deref()
                .map(|t| (t, self.x_axis.label_style)),
            self.y_axis
                .title
                .as_deref()
                .map(|t| (t, self.y_axis.label_style)),
        );
        // overlay legend (top-left of plot)
        let mut x = plot.x + 1;
        for ov in &self.overlays {
            let w = text_width(&ov.name) + 3;
            if x + w > plot.right() {
                break;
            }
            buf.set_string(x, plot.y, "●", Style::default().fg(ov.color));
            buf.set_string(x + 2, plot.y, &ov.name, Style::default().fg(Color::Gray));
            x += w + 1;
        }
    }
}

impl Widget for &CandlestickChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

impl Widget for CandlestickChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(n: usize) -> Vec<Candle> {
        (0..n)
            .map(|i| {
                let b = 100.0 + (i as f64 * 0.7).sin() * 10.0;
                let up = i % 2 == 0;
                let (o, c) = if up { (b, b + 3.0) } else { (b + 3.0, b) };
                Candle::new(o, b + 5.0, b - 2.0, c)
                    .volume(100.0 + i as f64 * 10.0)
                    .label(format!("{i}"))
            })
            .collect()
    }

    fn dump(buf: &Buffer, area: Rect) -> String {
        (area.y..area.bottom())
            .flat_map(|y| (area.x..area.right()).map(move |x| (x, y)))
            .map(|p| buf[p].symbol().to_string())
            .collect()
    }

    #[test]
    fn candle_new_widens_range() {
        let c = Candle::new(10.0, 9.0, 11.0, 12.0);
        assert_eq!(c.high, 12.0);
        assert_eq!(c.low, 10.0);
        assert!(c.is_bullish());
        assert!(!Candle::new(5.0, 6.0, 1.0, 2.0).is_bullish());
    }

    #[test]
    fn sma_values() {
        let c: Vec<Candle> = [1.0, 2.0, 3.0, 4.0]
            .iter()
            .map(|&p| Candle::new(p, p, p, p))
            .collect();
        let s = sma(&c, 2);
        assert_eq!(s, vec![None, Some(1.5), Some(2.5), Some(3.5)]);
        assert_eq!(sma(&c, 0).len(), 4);
    }

    #[test]
    fn ema_values() {
        let c: Vec<Candle> = [1.0, 2.0, 3.0, 4.0]
            .iter()
            .map(|&p| Candle::new(p, p, p, p))
            .collect();
        let e = ema(&c, 2);
        assert_eq!(e[0], None);
        assert_eq!(e[1], Some(1.5));
        let k = 2.0 / 3.0;
        assert!((e[2].unwrap() - (3.0 * k + 1.5 * (1.0 - k))).abs() < 1e-12);
    }

    #[test]
    fn heikin_ashi_first_bar() {
        let c = vec![Candle::new(10.0, 14.0, 8.0, 12.0)];
        let h = heikin_ashi(&c);
        assert_eq!(h[0].close, 11.0);
        assert_eq!(h[0].open, 11.0);
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn visible_range_scrolls() {
        let ch = CandlestickChart::new(data(20));
        assert_eq!(ch.visible_range(5), (15, 20));
        let ch = ch.scroll(3);
        assert_eq!(ch.visible_range(5), (12, 17));
        assert_eq!(CandlestickChart::new(data(3)).visible_range(10), (0, 3));
        assert_eq!(
            CandlestickChart::new(data(3)).scroll(10).visible_range(4),
            (0, 0)
        );
    }

    #[test]
    fn renders_all_styles() {
        for style in [
            CandleStyle::Candle,
            CandleStyle::Ohlc,
            CandleStyle::HeikinAshi,
        ] {
            let area = Rect::new(0, 0, 60, 20);
            let mut buf = Buffer::empty(area);
            let d = data(30);
            let s = sma(&d, 5);
            CandlestickChart::new(d)
                .style(style)
                .show_volume(true)
                .overlay(Overlay::new("SMA5", s, Color::Yellow))
                .block(Block::bordered())
                .render(area, &mut buf);
            let t = dump(&buf, area);
            assert!(t.contains('└'), "{style:?}");
            assert!(t.contains('●'), "{style:?}");
        }
    }

    #[test]
    fn candle_glyphs_present() {
        let area = Rect::new(0, 0, 40, 16);
        let mut buf = Buffer::empty(area);
        CandlestickChart::new(data(10))
            .candle_width(3)
            .render(area, &mut buf);
        let t = dump(&buf, area);
        assert!(t.contains('█'));
        assert!(t.contains('│'));
    }

    #[test]
    fn ohlc_glyphs_present() {
        let area = Rect::new(0, 0, 40, 16);
        let mut buf = Buffer::empty(area);
        CandlestickChart::new(data(10))
            .style(CandleStyle::Ohlc)
            .render(area, &mut buf);
        let t = dump(&buf, area);
        assert!(t.contains('┤') || t.contains('┼'));
        assert!(t.contains('├') || t.contains('┼'));
    }

    #[test]
    fn bullish_bearish_colors() {
        let area = Rect::new(0, 0, 20, 12);
        let mut buf = Buffer::empty(area);
        let c = vec![
            Candle::new(10.0, 20.0, 5.0, 18.0),
            Candle::new(18.0, 20.0, 5.0, 8.0),
        ];
        CandlestickChart::new(c)
            .colors(Color::Blue, Color::Magenta)
            .last_price_line(false)
            .render(area, &mut buf);
        let fgs: Vec<Color> = (0..20)
            .flat_map(|x| (0..12).map(move |y| (x, y)))
            .filter(|p| buf[*p].symbol() == "█")
            .map(|p| buf[p].fg)
            .collect();
        assert!(fgs.contains(&Color::Blue));
        assert!(fgs.contains(&Color::Magenta));
    }

    #[test]
    fn volume_panel_draws_eighths() {
        let area = Rect::new(0, 0, 40, 20);
        let mut buf = Buffer::empty(area);
        CandlestickChart::new(data(10))
            .show_volume(true)
            .render(area, &mut buf);
        let t = dump(&buf, area);
        assert!(t.contains('▇') || t.contains('█'));
    }

    #[test]
    fn handles_empty_and_invalid() {
        let area = Rect::new(0, 0, 30, 10);
        let mut buf = Buffer::empty(area);
        CandlestickChart::new(vec![]).render(area, &mut buf);
        CandlestickChart::new(vec![Candle {
            open: f64::NAN,
            high: 1.0,
            low: 0.0,
            close: 1.0,
            ..Default::default()
        }])
        .render(area, &mut buf);
    }

    #[test]
    fn single_flat_candle() {
        let area = Rect::new(0, 0, 30, 10);
        let mut buf = Buffer::empty(area);
        CandlestickChart::new(vec![Candle::new(5.0, 5.0, 5.0, 5.0)]).render(area, &mut buf);
    }

    #[test]
    fn tiny_areas_safe() {
        for (w, h) in [(0, 0), (3, 3), (8, 4), (12, 5)] {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            CandlestickChart::new(data(5))
                .show_volume(true)
                .render(area, &mut buf);
        }
    }
}
