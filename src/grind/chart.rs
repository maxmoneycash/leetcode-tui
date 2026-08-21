//! Terminal candlestick chart widget for the wpm series.
//!
//! Each candle takes one column (with a blank spacer column between candles):
//! `│` wicks, `█` body, green for bullish, red for bearish — the classic
//! look, in unicode.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Widget};

use super::candles::Candle;

pub struct CandleChart<'a> {
    candles: &'a [Candle],
    block: Option<Block<'a>>,
}

impl<'a> CandleChart<'a> {
    pub fn new(candles: &'a [Candle]) -> Self {
        Self {
            candles,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

const Y_LABEL_WIDTH: u16 = 5;

impl Widget for CandleChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match self.block {
            Some(b) => {
                let inner = b.inner(area);
                b.render(area, buf);
                inner
            }
            None => area,
        };
        if inner.width <= Y_LABEL_WIDTH + 2 || inner.height < 3 || self.candles.is_empty() {
            return;
        }

        let chart_x = inner.x + Y_LABEL_WIDTH;
        let chart_width = inner.width - Y_LABEL_WIDTH;
        let height = inner.height;

        // Fit the most recent candles into the width, one candle every 2 cols.
        let max_candles = (chart_width as usize).div_ceil(2);
        let start = self.candles.len().saturating_sub(max_candles);
        let visible = &self.candles[start..];

        let hi = visible.iter().map(|c| c.high).fold(f64::MIN, f64::max);
        let lo = visible.iter().map(|c| c.low).fold(f64::MAX, f64::min);
        // Give the scale a little headroom and never a zero range.
        let hi = hi.max(1.0) * 1.05;
        let lo = (lo * 0.95).max(0.0);
        let range = (hi - lo).max(1.0);

        let row_of = |v: f64| -> u16 {
            let frac = ((v - lo) / range).clamp(0.0, 1.0);
            // row 0 is the top of the chart
            let r = ((1.0 - frac) * (height.saturating_sub(1)) as f64).round() as u16;
            r.min(height - 1)
        };

        // y-axis labels: top, middle, bottom
        for (frac, label_v) in [(0.0f64, hi), (0.5, (hi + lo) / 2.0), (1.0, lo)] {
            let y = inner.y + ((height.saturating_sub(1)) as f64 * frac).round() as u16;
            let label = format!("{:>4.0}", label_v);
            buf.set_string(inner.x, y, label, Style::default().fg(Color::DarkGray));
        }

        for (i, c) in visible.iter().enumerate() {
            let x = chart_x + (i as u16) * 2;
            if x >= chart_x + chart_width {
                break;
            }
            let color = if c.is_bullish() {
                Color::Green
            } else {
                Color::Red
            };
            let top_wick = row_of(c.high);
            let bottom_wick = row_of(c.low);
            let body_top = row_of(c.open.max(c.close));
            let body_bottom = row_of(c.open.min(c.close));
            for row in top_wick..=bottom_wick {
                let sym = if row >= body_top && row <= body_bottom {
                    "█"
                } else {
                    "│"
                };
                buf.set_string(x, inner.y + row, sym, Style::default().fg(color));
            }
        }
    }
}
