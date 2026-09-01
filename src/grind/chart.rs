//! Terminal candlestick chart for the wpm series.
//!
//! Naive terminal candles draw one cell per row, so a candle can only ever
//! start and end on a row boundary and the result looks like a bar chart.
//! The fix is a glyph set whose characters carry *sub-cell* detail: box
//! drawing has heavy/light vertical pieces, and `╽` / `╿` render a thin wick
//! and a thick body inside a single cell. That doubles the effective
//! vertical resolution without needing a pixel protocol.
//!
//! The glyph table and the quarter-cell thresholds below are adapted from
//! Julien-R44's `cli-candlestick-chart` (MIT), which prints straight to
//! stdout; this is a ratatui widget so it can sit inside the app's layout.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Widget};

use super::candles::Candle;
use super::theme::theme;

/// Full-height body.
const BODY: &str = "┃";
/// Body filling only the bottom half of the cell.
const HALF_BODY_BOTTOM: &str = "╻";
/// Body filling only the top half of the cell.
const HALF_BODY_TOP: &str = "╹";
/// Full-height wick.
const WICK: &str = "│";
/// Wick above, body below, in one cell.
const WICK_OVER_BODY: &str = "╽";
/// Body above, wick below, in one cell.
const BODY_OVER_WICK: &str = "╿";
/// Wick occupying only the bottom half.
const WICK_BOTTOM: &str = "╷";
/// Wick occupying only the top half.
const WICK_TOP: &str = "╵";
/// A body too short to fill even half a cell.
const DOJI: &str = "─";
/// Bodies shorter than this (in chart rows) cannot be drawn as a body at all,
/// so they render as a doji mark.
const MIN_BODY_UNITS: f64 = 0.5;

/// Glyphs that represent body rather than wick. Bodies are drawn at the
/// candle's full width; wicks stay one column wide and centred.
fn is_body(g: &str) -> bool {
    matches!(
        g,
        BODY | HALF_BODY_BOTTOM | HALF_BODY_TOP | WICK_OVER_BODY | BODY_OVER_WICK | DOJI
    )
}

/// Columns reserved for price labels, e.g. "123.4".
const Y_LABEL_WIDTH: u16 = 5;
/// Label + gridline gutter.
const Y_AXIS_WIDTH: u16 = Y_LABEL_WIDTH + 1;
/// A price tick every N rows.
const TICK_EVERY: u16 = 3;

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

/// Pick the glyph for one candle at one row.
///
/// `y` is the row expressed in chart units with 0 at the bottom; the price
/// coordinates are fractional, and comparing them against quarter-cell
/// thresholds is what recovers the half-cell detail.
fn glyph_for(y: f64, high: f64, low: f64, body_top: f64, body_bottom: f64) -> Option<&'static str> {
    // A body shorter than the cell resolution. Handled up front: otherwise it
    // falls through to a thin wick glyph and reads as a gap in the run of
    // candles, and a candle flat in every component (a wpm plateau) matches no
    // branch at all and silently leaves a hole in the chart.
    if (body_top - body_bottom).abs() < MIN_BODY_UNITS && (body_top - y).abs() < 0.5 {
        return Some(DOJI);
    }

    if high.ceil() >= y && y >= body_top.floor() {
        // upper wick region, possibly transitioning into the body
        if body_top - y > 0.75 {
            Some(BODY)
        } else if body_top - y > 0.25 {
            if high - y > 0.75 {
                Some(WICK_OVER_BODY)
            } else {
                Some(HALF_BODY_BOTTOM)
            }
        } else if high - y > 0.75 {
            Some(WICK)
        } else if high - y > 0.25 {
            Some(WICK_BOTTOM)
        } else {
            None
        }
    } else if body_top.floor() >= y && y >= body_bottom.ceil() {
        Some(BODY)
    } else if body_bottom.ceil() >= y && y >= low.floor() {
        // lower wick region
        if body_bottom - y < 0.25 {
            Some(BODY)
        } else if body_bottom - y < 0.75 {
            if low - y < 0.25 {
                Some(BODY_OVER_WICK)
            } else {
                Some(HALF_BODY_TOP)
            }
        } else if low - y < 0.25 {
            Some(WICK)
        } else if low - y < 0.75 {
            Some(WICK_TOP)
        } else {
            None
        }
    } else {
        None
    }
}

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
        if inner.width <= Y_AXIS_WIDTH + 2 || inner.height < 3 {
            return;
        }

        let plot_x = inner.x + Y_AXIS_WIDTH;
        let plot_w = inner.width - Y_AXIS_WIDTH;
        let h = inner.height;

        // One candle per column, most recent flush against the right edge, so
        // the newest data sits where the eye already is.
        let visible: &[Candle] = if self.candles.len() > plot_w as usize {
            &self.candles[self.candles.len() - plot_w as usize..]
        } else {
            self.candles
        };

        let (mut hi, mut lo) = (f64::MIN, f64::MAX);
        for c in visible {
            hi = hi.max(c.high);
            lo = lo.min(c.low);
        }
        if visible.is_empty() {
            hi = 1.0;
            lo = 0.0;
        }
        // Pad so candles never touch the frame, and never allow a zero range.
        let pad = ((hi - lo) * 0.08).max(1.0);
        let (hi, lo) = (hi + pad, (lo - pad).max(0.0));
        let range = (hi - lo).max(1e-9);

        let rows = (h - 1) as f64;
        let to_unit = |price: f64| ((price - lo) / range * rows).clamp(0.0, rows);

        // Recessive axis: dim labels, dotted gridline. The data is the only
        // thing that should carry weight.
        let th = theme();
        let axis_style = Style::default().fg(th.rule);
        for r in 0..h {
            let y = inner.y + r;
            if r % TICK_EVERY == 0 {
                let unit = (h - 1 - r) as f64;
                let price = lo + unit / rows * range;
                buf.set_string(
                    inner.x,
                    y,
                    format!("{:>width$.0}", price, width = Y_LABEL_WIDTH as usize),
                    axis_style,
                );
                buf.set_string(inner.x + Y_LABEL_WIDTH, y, "┈", axis_style);
            } else {
                buf.set_string(inner.x + Y_LABEL_WIDTH, y, "╎", axis_style);
            }
        }

        // Candles widen when there are few of them, the way a real chart does,
        // instead of leaving most of the plot empty. `unit` is the pitch
        // (body + one column of gap).
        let n = visible.len().max(1);
        let unit = ((plot_w as usize) / n).clamp(1, 5) as u16;
        let body_w = unit.saturating_sub(1).max(1);

        // Anchor left and grow rightward. A scrolling ticker pins the newest
        // candle to the right edge, but this chart starts empty at the top of
        // a run: right-anchoring would leave a void on the left where history
        // would be, which reads as broken. Growing rightward reads as a chart
        // being drawn, and once the candles overflow the pane the slice above
        // keeps only the most recent, so it behaves like a ticker anyway.
        let start_x = plot_x;

        for (i, c) in visible.iter().enumerate() {
            let x0 = start_x + i as u16 * unit;
            if x0 >= plot_x + plot_w {
                break;
            }
            let wick_x = x0 + (body_w - 1) / 2;
            let style = Style::default().fg(if c.is_bullish() { th.bull } else { th.bear });
            let high = to_unit(c.high);
            let low = to_unit(c.low);
            let body_top = to_unit(c.open.max(c.close));
            let body_bottom = to_unit(c.open.min(c.close));

            for r in 0..h {
                let y = (h - 1 - r) as f64;
                let Some(g) = glyph_for(y, high, low, body_top, body_bottom) else {
                    continue;
                };
                let row = inner.y + r;
                if is_body(g) {
                    for dx in 0..body_w {
                        let x = x0 + dx;
                        if x < plot_x + plot_w {
                            buf.set_string(x, row, g, style);
                        }
                    }
                } else if wick_x < plot_x + plot_w {
                    buf.set_string(wick_x, row, g, style);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_region_is_solid() {
        // body spans units 2..=4, wicks out to 0 and 6
        assert_eq!(glyph_for(3.0, 6.0, 0.0, 4.0, 2.0), Some(BODY));
    }

    #[test]
    fn wick_region_is_thin() {
        assert_eq!(glyph_for(5.0, 6.0, 0.0, 4.0, 2.0), Some(WICK));
        assert_eq!(glyph_for(1.0, 6.0, 0.0, 4.0, 2.0), Some(WICK));
    }

    #[test]
    fn outside_the_candle_is_empty() {
        assert_eq!(glyph_for(9.0, 6.0, 0.0, 4.0, 2.0), None);
    }

    #[test]
    fn half_cell_transitions_are_used() {
        // a body edge landing mid-cell must produce a transition glyph
        // rather than snapping to a full body or a bare wick
        let g = glyph_for(4.0, 6.0, 0.0, 4.5, 2.0);
        assert!(
            g == Some(WICK_OVER_BODY) || g == Some(HALF_BODY_BOTTOM),
            "expected a half-cell glyph, got {:?}",
            g
        );
    }

    fn render_to(candles: &[Candle], w: u16, h: u16) -> Buffer {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        CandleChart::new(candles).render(area, &mut buf);
        buf
    }

    fn row_text(buf: &Buffer, y: u16, w: u16) -> String {
        (0..w).map(|x| buf.get(x, y).symbol.clone()).collect()
    }

    #[test]
    fn candles_grow_from_the_left() {
        let c = Candle {
            open: 10.0,
            high: 20.0,
            low: 5.0,
            close: 15.0,
        };
        let buf = render_to(&[c, c], 60, 12);
        // ink must start just right of the axis gutter, not floating at the
        // right edge with a hole where history would be
        let first_col: String = (0..12)
            .map(|y| buf.get(Y_AXIS_WIDTH, y).symbol.clone())
            .collect();
        assert!(
            first_col.chars().any(|ch| ch != ' '),
            "expected candles anchored at the left of the plot, got {:?}",
            first_col
        );
        // with only 2 candles the far right stays empty
        let right_col: String = (0..12).map(|y| buf.get(59, y).symbol.clone()).collect();
        assert!(
            right_col.trim().is_empty(),
            "expected empty space to the right, got {:?}",
            right_col
        );
    }

    #[test]
    fn few_candles_get_wide_bodies() {
        let c = Candle {
            open: 10.0,
            high: 20.0,
            low: 5.0,
            close: 15.0,
        };
        let buf = render_to(&[c, c], 60, 12);
        // with 2 candles across 60 columns the body must be more than one
        // column wide, otherwise the chart is mostly whitespace
        let widest = (0..12)
            .map(|y| {
                row_text(&buf, y, 60)
                    .chars()
                    .filter(|ch| *ch == '┃')
                    .count()
            })
            .max()
            .unwrap();
        assert!(
            widest >= 2,
            "expected widened bodies, widest row had {}",
            widest
        );
    }

    #[test]
    fn many_candles_still_fit() {
        let c = Candle {
            open: 10.0,
            high: 20.0,
            low: 5.0,
            close: 15.0,
        };
        let many = vec![c; 500];
        // must not panic or overflow the buffer
        let buf = render_to(&many, 40, 10);
        assert_eq!(buf.area().width, 40);
    }

    #[test]
    fn doji_still_renders() {
        // open == close == high == low must not vanish
        assert_eq!(glyph_for(3.0, 3.0, 3.0, 3.0, 3.0), Some(DOJI));
    }

    #[test]
    fn sub_cell_body_is_a_doji_not_a_wick() {
        // a body 0.3 rows tall cannot be drawn as a body; it must still read
        // as a candle rather than as a bare wick
        assert_eq!(glyph_for(3.0, 6.0, 0.0, 3.15, 2.85), Some(DOJI));
    }

    #[test]
    fn flat_body_with_wicks_is_a_doji_not_a_wick() {
        // open == close but the candle still has range: the body row must
        // read as a doji mark, and the wicks above/below still draw
        assert_eq!(glyph_for(3.0, 6.0, 0.0, 3.0, 3.0), Some(DOJI));
        assert_eq!(glyph_for(5.0, 6.0, 0.0, 3.0, 3.0), Some(WICK));
    }
}
