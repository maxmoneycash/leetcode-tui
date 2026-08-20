//! Aggregates rolling-wpm samples into fixed-period OHLC candles, so a
//! typing run charts like a market ticker.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Candle {
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }
}

#[derive(Debug)]
pub struct CandleSeries {
    period_secs: f64,
    /// Completed candles.
    pub candles: Vec<Candle>,
    /// Candle currently forming, with the bucket index it belongs to.
    current: Option<(usize, Candle)>,
}

impl CandleSeries {
    pub fn new(period_secs: f64) -> Self {
        Self {
            period_secs,
            candles: vec![],
            current: None,
        }
    }

    /// Feed one wpm sample taken at `t_secs` since the run started.
    pub fn push_sample(&mut self, t_secs: f64, wpm: f64) {
        let bucket = (t_secs / self.period_secs).floor().max(0.0) as usize;
        match &mut self.current {
            Some((cur_bucket, candle)) if *cur_bucket == bucket => {
                candle.high = candle.high.max(wpm);
                candle.low = candle.low.min(wpm);
                candle.close = wpm;
            }
            Some((_, candle)) => {
                let prev_close = candle.close;
                self.candles.push(*candle);
                self.current = Some((
                    bucket,
                    Candle {
                        // a candle opens where the previous one closed
                        open: prev_close,
                        high: prev_close.max(wpm),
                        low: prev_close.min(wpm),
                        close: wpm,
                    },
                ));
            }
            None => {
                self.current = Some((
                    bucket,
                    Candle {
                        open: wpm,
                        high: wpm,
                        low: wpm,
                        close: wpm,
                    },
                ));
            }
        }
    }

    /// Completed candles plus the one still forming.
    pub fn all(&self) -> Vec<Candle> {
        let mut out = self.candles.clone();
        if let Some((_, c)) = &self.current {
            out.push(*c);
        }
        out
    }

    pub fn session_high(&self) -> f64 {
        self.all().iter().map(|c| c.high).fold(0.0, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ohlc_within_a_bucket() {
        let mut s = CandleSeries::new(2.0);
        s.push_sample(0.1, 40.0);
        s.push_sample(0.5, 55.0);
        s.push_sample(1.9, 45.0);
        let all = s.all();
        assert_eq!(all.len(), 1);
        let c = all[0];
        assert_eq!(c.open, 40.0);
        assert_eq!(c.high, 55.0);
        assert_eq!(c.low, 40.0);
        assert_eq!(c.close, 45.0);
    }

    #[test]
    fn new_bucket_opens_at_previous_close() {
        let mut s = CandleSeries::new(2.0);
        s.push_sample(0.5, 40.0);
        s.push_sample(1.5, 60.0);
        s.push_sample(2.5, 50.0);
        let all = s.all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].close, 60.0);
        assert_eq!(all[1].open, 60.0);
        assert_eq!(all[1].low, 50.0);
        assert!(!all[1].is_bullish());
    }

    #[test]
    fn session_high_spans_all_candles() {
        let mut s = CandleSeries::new(1.0);
        s.push_sample(0.5, 30.0);
        s.push_sample(1.5, 80.0);
        s.push_sample(2.5, 20.0);
        assert_eq!(s.session_high(), 80.0);
    }
}
