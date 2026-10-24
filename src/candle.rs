//! Candlestick / OHLC chart for price data (stocks, crypto, FX…).

use crate::axis::Axis;
use crate::canvas::{Canvas, Marker};
use crate::indicator;
use crate::layout::{self, text_width};
use crate::theme::{self, Theme};
use crate::util::{format_compact, nice_bounds};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Block,
};

/// One OHLC(V) bar.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    setter!(
        /// Attach volume.
        volume: f64
    );

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CandleStyle {
    /// Japanese candlesticks with body and wicks.
    #[default]
    Candle,
    /// OHLC bars: vertical range with open (left) / close (right) ticks.
    Ohlc,
    /// Heikin-Ashi smoothed candles.
    HeikinAshi,
    /// High-resolution braille candles: 1-dot wicks and 2-dot-wide bodies
    /// (per cell of body width) with 4 dots of vertical resolution per row.
    /// Bodies sit edge to edge, so about twice as many candles fit per width
    /// (`gap(1)`, the default, means no gap; `gap(2)` leaves one cell).
    Braille,
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

/// Shaded band between two series (e.g. Bollinger bands) aligned to the candle index.
#[derive(Debug, Clone)]
pub struct BandOverlay {
    /// Legend name.
    pub name: String,
    /// Upper bound per candle.
    pub upper: Vec<Option<f64>>,
    /// Lower bound per candle.
    pub lower: Vec<Option<f64>>,
    /// Shading color.
    pub color: Color,
}

impl BandOverlay {
    /// Create a band overlay.
    pub fn new(
        name: impl Into<String>,
        upper: Vec<Option<f64>>,
        lower: Vec<Option<f64>>,
        color: Color,
    ) -> Self {
        Self {
            name: name.into(),
            upper,
            lower,
            color,
        }
    }

    /// Bollinger band (`period`, `k` standard deviations) over `candles`.
    pub fn bollinger(candles: &[Candle], period: usize, k: f64, color: Color) -> Self {
        let (_, up, lo) = indicator::bollinger(candles, period, k);
        Self::new(format!("BB{period}"), up, lo, color)
    }
}

/// Sub-pane drawn below the price (and volume) panel, sharing the x axis.
#[derive(Debug, Clone)]
pub enum IndicatorPane {
    /// RSI with 30/70 reference lines.
    Rsi {
        /// Lookback period (commonly 14).
        period: usize,
    },
    /// MACD lines plus histogram bars.
    Macd {
        /// Fast EMA period (commonly 12).
        fast: usize,
        /// Slow EMA period (commonly 26).
        slow: usize,
        /// Signal EMA period (commonly 9).
        signal: usize,
    },
    /// Arbitrary line series, auto-scaled.
    Custom {
        /// Pane label.
        name: String,
        /// One value per candle.
        values: Vec<Option<f64>>,
        /// Line color.
        color: Color,
    },
}

/// Side of a [`TradeMarker`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TradeSide {
    /// Buy (drawn as ▲ in the up color).
    Buy,
    /// Sell (drawn as ▼ in the down color).
    Sell,
}

/// A buy/sell marker placed at a candle index and price.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TradeMarker {
    /// Index into the full candle list.
    pub index: usize,
    /// Price level of the marker.
    pub price: f64,
    /// Buy or sell.
    pub side: TradeSide,
    /// Optional text drawn next to the glyph.
    pub label: Option<String>,
}

impl TradeMarker {
    /// Create a marker without label.
    pub fn new(index: usize, price: f64, side: TradeSide) -> Self {
        Self {
            index,
            price,
            side,
            label: None,
        }
    }

    /// Attach a label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Horizontal reference line (support / resistance / target…).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HLine {
    /// Price level.
    pub price: f64,
    /// Optional label (defaults to the price).
    pub label: Option<String>,
    /// Line color.
    pub color: Color,
}

impl HLine {
    /// Create a line at `price`.
    pub fn new(price: f64, color: Color) -> Self {
        Self {
            price,
            label: None,
            color,
        }
    }

    /// Attach a label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
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

fn hline_row(buf: &mut Buffer, area: Rect, row: u16, ch: char, style: Style) {
    for x in area.x..area.right() {
        if let Some(cell) = buf.cell_mut((x, area.y + row)) {
            if cell.symbol() == " " || cell.symbol() == "·" || cell.symbol() == "░" {
                cell.set_char(ch).set_style(style);
            }
        }
    }
}

/// Right-aligned text tag on a row of `area`.
fn tag(buf: &mut Buffer, area: Rect, row: u16, text: &str, style: Style) {
    let w = text_width(text);
    if w + 2 > area.width {
        return;
    }
    buf.set_string(area.right() - w, area.y + row, text, style);
}

const EIGHTHS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Candlestick / OHLC chart with optional volume panel and overlays.
///
/// # Example
///
/// ```
/// use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
/// use tui_chart::{Candle, CandlestickChart};
///
/// let chart = CandlestickChart::new(vec![Candle::new(10.0, 12.0, 9.0, 11.0), Candle::new(11.0, 13.0, 10.0, 12.5)]);
/// let area = Rect::new(0, 0, 40, 12);
/// let mut buf = Buffer::empty(area);
/// chart.render(area, &mut buf);
/// ```
#[derive(Debug, Clone)]
pub struct CandlestickChart {
    candles: Vec<Candle>,
    style: CandleStyle,
    colors: Option<(Color, Color)>,
    hollow: bool,
    theme: Option<Theme>,
    base: Option<Style>,
    candle_width: u16,
    gap: u16,
    show_volume: bool,
    last_price_line: bool,
    scroll: usize,
    overlays: Vec<Overlay>,
    bands: Vec<BandOverlay>,
    panes: Vec<IndicatorPane>,
    markers: Vec<TradeMarker>,
    hlines: Vec<HLine>,
    volume_profile: bool,
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
            colors: None,
            hollow: false,
            theme: None,
            base: None,
            candle_width: 1,
            gap: 1,
            show_volume: false,
            last_price_line: true,
            scroll: 0,
            overlays: vec![],
            bands: vec![],
            panes: vec![],
            markers: vec![],
            hlines: vec![],
            volume_profile: false,
            block: None,
            x_axis: Axis::new().ticks(6),
            y_axis: Axis::new().grid(true),
        }
    }

    setter!(
        /// Bar style.
        style: CandleStyle
    );

    /// Colors for bullish / bearish bars.
    pub fn colors(mut self, up: Color, down: Color) -> Self {
        self.colors = Some((up, down));
        self
    }

    setter!(
        /// Draw bullish candle bodies hollow (outlined) and bearish ones filled.
        hollow: bool
    );

    /// Apply a [`Theme`]: bull/bear colors, axes, labels, background fill.
    /// Colors set via [`CandlestickChart::colors`] still win.
    #[cfg(feature = "theme")]
    pub fn theme(mut self, theme: impl Into<Theme>) -> Self {
        let t = theme.into();
        self.x_axis.theme_in_place(&t);
        self.y_axis.theme_in_place(&t);
        self.theme = Some(t);
        self
    }

    base_style_setter!(base);

    fn up_color(&self) -> Color {
        match (&self.colors, &self.theme) {
            (Some((u, _)), _) => *u,
            (None, Some(t)) => t.bull,
            _ => Color::Green,
        }
    }

    fn down_color(&self) -> Color {
        match (&self.colors, &self.theme) {
            (Some((_, d)), _) => *d,
            (None, Some(t)) => t.bear,
            _ => Color::Red,
        }
    }

    fn dim_color(&self) -> Color {
        self.theme.as_ref().map_or(Color::DarkGray, |t| t.axis)
    }

    fn label_color(&self) -> Color {
        self.theme.as_ref().map_or(Color::Gray, |t| t.muted)
    }

    /// Body width in cells (ignored for OHLC bars; min 1).
    pub fn candle_width(mut self, w: u16) -> Self {
        self.candle_width = w.max(1);
        self
    }

    setter!(
        /// Empty cells between bars.
        gap: u16
    );

    setter!(
        /// Show a volume histogram under the price panel.
        show_volume: bool
    );

    setter!(
        /// Dashed horizontal line at the last visible close.
        last_price_line: bool
    );

    setter!(
        /// Hide the newest `n` candles (scroll back in time).
        scroll: usize
    );

    /// Add an indicator overlay.
    pub fn overlay(mut self, overlay: Overlay) -> Self {
        self.overlays.push(overlay);
        self
    }

    /// Add a shaded band (e.g. Bollinger) behind the candles.
    pub fn band(mut self, band: BandOverlay) -> Self {
        self.bands.push(band);
        self
    }

    /// Add an indicator sub-pane below price/volume (dropped if there is no room).
    pub fn pane(mut self, pane: IndicatorPane) -> Self {
        self.panes.push(pane);
        self
    }

    /// Add a buy/sell marker.
    pub fn marker(mut self, marker: TradeMarker) -> Self {
        self.markers.push(marker);
        self
    }

    /// Add a horizontal reference line.
    pub fn hline(mut self, line: HLine) -> Self {
        self.hlines.push(line);
        self
    }

    /// Support level (green dashed line labelled "S").
    pub fn support(self, price: f64) -> Self {
        self.hline(HLine::new(price, Color::Green).label(format!("S {price:.2}")))
    }

    /// Resistance level (red dashed line labelled "R").
    pub fn resistance(self, price: f64) -> Self {
        self.hline(HLine::new(price, Color::Red).label(format!("R {price:.2}")))
    }

    setter!(
        /// Horizontal volume-by-price histogram along the right of the price panel.
        volume_profile: bool
    );

    block_setter!(block, 'static);

    /// X axis (title / label style; labels come from [`Candle::label`]).
    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = axis;
        if let Some(t) = &self.theme {
            self.x_axis.theme_in_place(t);
        }
        self
    }

    /// Y (price) axis.
    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = axis;
        if let Some(t) = &self.theme {
            self.y_axis.theme_in_place(t);
        }
        self
    }

    /// Cells per candle slot (body + gap). Braille packs bodies edge to edge.
    fn slot_cells(&self) -> usize {
        let gap = if self.style == CandleStyle::Braille {
            self.gap.saturating_sub(1)
        } else {
            self.gap
        };
        (self.bar_width() + gap) as usize
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
            self.up_color()
        } else {
            self.down_color()
        }
    }

    fn draw_profile(
        &self,
        buf: &mut Buffer,
        price: Rect,
        vis: &[&Candle],
        sub: &dyn Fn(f64) -> i32,
    ) {
        let mut bins = vec![0.0f64; price.height as usize];
        for c in vis.iter().filter(|c| c.is_valid() && c.volume.is_finite()) {
            let (r0, r1) = ((sub(c.high) / 2) as usize, (sub(c.low) / 2) as usize);
            let share = c.volume / (r1 - r0 + 1) as f64;
            for b in bins.iter_mut().take(r1 + 1).skip(r0) {
                *b += share;
            }
        }
        let max = bins.iter().cloned().fold(0.0, f64::max);
        if max <= 0.0 {
            return;
        }
        let w = (price.width / 5).clamp(1, 20) as f64;
        let st = Style::default().fg(self.dim_color());
        for (r, v) in bins.iter().enumerate() {
            let len = (v / max * w).round() as u16;
            for dx in 0..len {
                buf.set_string(price.right() - 1 - dx, price.y + r as u16, "▒", st);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_pane(
        &self,
        buf: &mut Buffer,
        r: Rect,
        gutter: Rect,
        pane: &IndicatorPane,
        start: usize,
        n_vis: usize,
        slot: usize,
        bw: u16,
    ) {
        if r.height == 0 || r.width == 0 {
            return;
        }
        let dim = Style::default().fg(self.dim_color());
        // (name, lines [(values,color)], hist, bounds, refs)
        let (name, lines, hist, (lo, hi), refs): (String, Vec<_>, Option<Vec<_>>, _, Vec<f64>) =
            match pane {
                IndicatorPane::Rsi { period } => (
                    format!("RSI {period}"),
                    vec![(indicator::rsi(&self.candles, *period), Color::Cyan)],
                    None,
                    (0.0, 100.0),
                    vec![70.0, 30.0],
                ),
                IndicatorPane::Macd { fast, slow, signal } => {
                    let (m, s, h) = indicator::macd(&self.candles, *fast, *slow, *signal);
                    let range = (start..start + n_vis)
                        .flat_map(|i| [m.get(i), s.get(i), h.get(i)])
                        .flatten()
                        .flatten()
                        .fold(0.0f64, |a, v| a.max(v.abs()));
                    let range = if range > 0.0 { range } else { 1.0 };
                    (
                        format!("MACD {fast},{slow},{signal}"),
                        vec![(m, Color::Cyan), (s, Color::Yellow)],
                        Some(h),
                        (-range, range),
                        vec![0.0],
                    )
                }
                IndicatorPane::Custom {
                    name,
                    values,
                    color,
                } => {
                    let vals = || {
                        (start..start + n_vis)
                            .filter_map(|i| values.get(i).copied().flatten())
                            .filter(|v| v.is_finite())
                    };
                    let lo = vals().fold(f64::INFINITY, f64::min);
                    let hi = vals().fold(f64::NEG_INFINITY, f64::max);
                    let b = if hi > lo { (lo, hi) } else { (0.0, 1.0) };
                    (
                        name.clone(),
                        vec![(values.clone(), *color)],
                        None,
                        b,
                        vec![],
                    )
                }
            };
        let h = r.height as f64;
        let row = |v: f64| -> u16 {
            ((hi - v) / (hi - lo) * (h - 1.0))
                .round()
                .clamp(0.0, h - 1.0) as u16
        };
        let mut ticks = vec![];
        for rv in &refs {
            let rr = row(*rv);
            hline_row(buf, r, rr, '┄', dim);
            ticks.push((rr, format!("{rv:.0}")));
        }
        layout::draw_y_labels(buf, gutter, &ticks, dim);
        let xs = |i: usize| r.x + (i * slot) as u16;
        if let Some(hs) = &hist {
            let z = row(0.0);
            for i in 0..n_vis {
                let Some(Some(v)) = hs.get(start + i) else {
                    continue;
                };
                if !v.is_finite() {
                    continue;
                }
                let col = if *v >= 0.0 {
                    self.up_color()
                } else {
                    self.down_color()
                };
                let rv = row(*v);
                for dx in 0..bw {
                    let x = xs(i) + dx;
                    if x >= r.right() {
                        break;
                    }
                    for y in z.min(rv)..=z.max(rv) {
                        buf.set_string(x, r.y + y, "█", Style::default().fg(col));
                    }
                }
            }
        }
        for (vals, col) in &lines {
            for i in 0..n_vis {
                let Some(Some(v)) = vals.get(start + i) else {
                    continue;
                };
                let x = xs(i) + bw / 2;
                if !v.is_finite() || x >= r.right() {
                    continue;
                }
                buf.set_string(x, r.y + row(*v), "●", Style::default().fg(*col));
            }
        }
        if text_width(&name) < r.width {
            buf.set_string(r.x + 1, r.y, &name, dim);
        }
    }

    fn render_inner(&self, area: Rect, buf: &mut Buffer) {
        let inner = theme::begin(&self.block, &self.theme, self.base, area, buf);
        let src: Vec<Candle> = match self.style {
            CandleStyle::HeikinAshi => heikin_ashi(&self.candles),
            _ => self.candles.clone(),
        };
        let bw = self.bar_width();
        let slot = self.slot_cells();
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
            self.visible_range((plot.width as usize + slot - bw as usize) / slot.max(1));
        let vis: Vec<&Candle> = src[start.min(src.len())..end.min(src.len())]
            .iter()
            .collect();

        let vol_h = if self.show_volume && plot.height >= 8 {
            (plot.height / 4).max(2)
        } else {
            0
        };
        let pane_each = (plot.height / 6).max(3);
        let mut panes_n = 0usize;
        let mut panes_h = 0u16;
        for _ in &self.panes {
            let nh = panes_h + pane_each;
            if plot.height.saturating_sub(nh + vol_h) < 5 {
                break;
            }
            panes_n += 1;
            panes_h = nh;
        }
        let vol_bottom = plot.bottom() - panes_h;
        let price = Rect::new(plot.x, plot.y, plot.width, plot.height - vol_h - panes_h);
        let sub_n = price.height as f64 * 2.0;
        let sub = |p: f64| -> i32 {
            ((hi - p) / (hi - lo) * (sub_n - 1.0))
                .round()
                .clamp(0.0, sub_n - 1.0) as i32
        };

        if crate::interact::recording() {
            for (i, c) in vis.iter().enumerate().filter(|(_, c)| c.is_valid()) {
                let x = price.x + (i * slot) as u16;
                let (t, b) = ((sub(c.high) / 2) as u16, (sub(c.low) / 2) as u16);
                let rect =
                    Rect::new(x, price.y + t, bw.max(1), b.max(t) - t + 1).intersection(price);
                crate::interact::record_point(rect, 0, start + i, || {
                    let f = crate::util::format_compact;
                    let mut lines = vec![
                        format!("O {}  H {}", f(c.open), f(c.high)),
                        format!("L {}  C {}", f(c.low), f(c.close)),
                    ];
                    if c.volume > 0.0 {
                        lines.push(format!("vol {}", f(c.volume)));
                    }
                    (
                        c.label.clone().unwrap_or_else(|| format!("#{}", start + i)),
                        lines,
                    )
                });
            }
        }

        // grid + y labels
        let y_ticks: Vec<(u16, String)> = y_marks
            .iter()
            .map(|(f, l)| (layout::frac_row(*f, price.height, 2), l.clone()))
            .collect();
        if self.y_axis.grid {
            let rows: Vec<u16> = y_ticks.iter().map(|t| t.0).collect();
            layout::draw_grid(
                buf,
                price,
                &[],
                &rows,
                self.y_axis.grid_style,
                self.y_axis.grid_kind,
            );
        }

        // volume profile + bands (behind candles)
        if self.volume_profile && price.width >= 20 {
            self.draw_profile(buf, price, &vis, &sub);
        }
        for band in &self.bands {
            let st = Style::default().fg(band.color);
            for i in 0..vis.len() {
                let (Some(Some(u)), Some(Some(l))) =
                    (band.upper.get(start + i), band.lower.get(start + i))
                else {
                    continue;
                };
                if !u.is_finite() || !l.is_finite() || (*u < lo && *l < lo) || (*u > hi && *l > hi)
                {
                    continue;
                }
                let x0 = price.x + (i * slot) as u16;
                for dx in 0..slot.min(2).max(bw as usize) as u16 {
                    let x = x0 + dx;
                    if x >= price.right() {
                        break;
                    }
                    for r in (sub(*u) / 2)..=(sub(*l) / 2) {
                        if let Some(cell) = buf.cell_mut((x, price.y + r as u16)) {
                            if cell.symbol() == " " || cell.symbol() == "·" {
                                cell.set_char('░').set_style(st);
                            }
                        }
                    }
                }
            }
        }

        // horizontal lines + last price line
        for h in &self.hlines {
            if h.price.is_finite() && h.price >= lo && h.price <= hi {
                let row = (sub(h.price) / 2) as u16;
                let label = h.label.clone().unwrap_or_else(|| format!("{:.2}", h.price));
                hline_row(buf, price, row, '┄', Style::default().fg(h.color));
                tag(buf, price, row, &label, Style::default().fg(h.color));
            }
        }
        let last_tag = if self.last_price_line {
            vis.iter().rev().find(|c| c.is_valid()).map(|c| {
                let row = (sub(c.close) / 2) as u16;
                hline_row(buf, price, row, '┈', Style::default().fg(self.color_of(c)));
                (row, format!("{:.2}", c.close), self.color_of(c))
            })
        } else {
            None
        };

        let max_vol = vis
            .iter()
            .map(|c| c.volume)
            .filter(|v| v.is_finite())
            .fold(0.0f64, f64::max);
        let mut x_ticks: Vec<(u16, String)> = vec![];
        let mut dots = (self.style == CandleStyle::Braille)
            .then(|| Canvas::new(price.width, price.height, Marker::Braille));
        let dot_n = price.height as f64 * 4.0;
        let dsub = |p: f64| -> i32 {
            ((hi - p) / (hi - lo) * (dot_n - 1.0))
                .round()
                .clamp(0.0, dot_n - 1.0) as i32
        };
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
            if let Some(cv) = dots.as_mut() {
                let px0 = i32::from(x0 - price.x) * 2;
                let pw = i32::from(bw) * 2;
                let wick = px0 + (pw - 1) / 2;
                let (w0, w1) = (dsub(c.high), dsub(c.low));
                for py in w0..=w1 {
                    cv.set(wick, py, color);
                }
                let (oa, ob) = (dsub(c.open), dsub(c.close));
                let (b0, b1) = (oa.min(ob), oa.max(ob));
                let hollow = self.hollow && c.is_bullish();
                for py in b0..=b1 {
                    for px in px0..px0 + pw {
                        let edge = py == b0 || py == b1 || px == px0 || px == px0 + pw - 1;
                        if !hollow || edge {
                            cv.set(px, py, color);
                        }
                    }
                }
            } else if self.style == CandleStyle::Ohlc {
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
                    let hollow = self.hollow && c.is_bullish();
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
                        let ch = match ch {
                            Some('█') if hollow => {
                                let (top_r, bot_r) = (b0 / 2, b1 / 2);
                                if bw == 1 {
                                    Some('┃')
                                } else if dx == 0 {
                                    Some('▌')
                                } else if dx == bw - 1 {
                                    Some('▐')
                                } else if r == top_r {
                                    Some('▔')
                                } else if r == bot_r {
                                    Some('▁')
                                } else {
                                    Some(' ')
                                }
                            }
                            other => other,
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
                    let y = vol_bottom - 1 - r as u16;
                    for dx in 0..bw {
                        buf.set_string(x0 + dx, y, EIGHTHS[fill].to_string(), st);
                    }
                }
            }
        }

        if let Some(cv) = &dots {
            cv.render(buf, price);
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

        // trade markers
        for m in &self.markers {
            if m.index < start || m.index >= start + vis.len() || !m.price.is_finite() {
                continue;
            }
            if m.price < lo || m.price > hi {
                continue;
            }
            let i = m.index - start;
            let x = price.x + (i * slot) as u16 + bw / 2;
            if x >= price.right() {
                continue;
            }
            let row = (sub(m.price) / 2) as u16;
            let (g, col) = match m.side {
                TradeSide::Buy => ('▲', self.up_color()),
                TradeSide::Sell => ('▼', self.down_color()),
            };
            let st = Style::default().fg(col).add_modifier(Modifier::BOLD);
            buf.set_string(x, price.y + row, g.to_string(), st);
            if let Some(l) = &m.label {
                if x + 1 + text_width(l) < price.right() {
                    buf.set_string(x + 1, price.y + row, l, Style::default().fg(col));
                }
            }
        }
        if let Some((row, text, col)) = last_tag {
            let st = Style::default().fg(Color::Black).bg(col);
            tag(buf, price, row, &format!(" {text} "), st);
        }

        // indicator panes
        for (k, pane) in self.panes.iter().take(panes_n).enumerate() {
            let r = Rect::new(
                plot.x,
                plot.bottom() - panes_h + k as u16 * pane_each,
                plot.width,
                pane_each,
            );
            let gutter = Rect::new(a.y_labels.x, r.y, a.y_labels.width, r.height);
            self.draw_pane(buf, r, gutter, pane, start, vis.len(), slot, bw);
        }

        layout::draw_axes(buf, &a, self.x_axis.style, self.y_axis.style);
        layout::draw_y_labels(
            buf,
            Rect::new(a.y_labels.x, price.y, a.y_labels.width, price.height),
            &y_ticks,
            self.y_axis.label_style,
        );
        if vol_h > 0 {
            let top = Rect::new(a.y_labels.x, vol_bottom - vol_h, a.y_labels.width, vol_h);
            let label = format_compact(max_vol);
            layout::draw_y_labels(
                buf,
                top,
                &[(0, label)],
                Style::default().fg(self.dim_color()),
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
            buf.set_string(
                x + 2,
                plot.y,
                &ov.name,
                Style::default().fg(self.label_color()),
            );
            x += w + 1;
        }
    }
}

impl CandlestickChart {
    fn render_into(&self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

widget_impls!(CandlestickChart);

#[cfg(feature = "theme")]
impl tui_nuances::Themed for CandlestickChart {
    fn apply_theme(self, theme: &tui_nuances::Theme) -> Self {
        self.theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Widget;

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
            CandleStyle::Braille,
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

    #[cfg(feature = "theme")]
    #[test]
    fn braille_candles_dense_and_colored() {
        let area = Rect::new(0, 0, 40, 16);
        let braille_cols = |style| {
            let mut buf = Buffer::empty(area);
            CandlestickChart::new(data(30))
                .style(style)
                .colors(Color::Green, Color::Red)
                .last_price_line(false)
                .render(area, &mut buf);
            let mut cols = std::collections::HashSet::new();
            let mut colors = std::collections::HashSet::new();
            for y in 0..16 {
                for x in 0..40 {
                    let c = &buf[(x, y)];
                    let ch = c.symbol().chars().next().unwrap();
                    if ('\u{2801}'..='\u{28FF}').contains(&ch) {
                        cols.insert(x);
                        colors.insert(c.fg);
                    }
                }
            }
            (cols.len(), colors)
        };
        let (n, colors) = braille_cols(CandleStyle::Braille);
        assert!(n >= 25, "{n}");
        assert!(colors.contains(&Color::Green) || colors.contains(&Color::Red));
        assert_eq!(braille_cols(CandleStyle::Candle).0, 0);
        // themed + hollow + gap + volume do not panic
        let mut buf = Buffer::empty(area);
        CandlestickChart::new(data(30))
            .style(CandleStyle::Braille)
            .theme(Theme::nord())
            .hollow(true)
            .gap(2)
            .candle_width(2)
            .show_volume(true)
            .render(area, &mut buf);
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
