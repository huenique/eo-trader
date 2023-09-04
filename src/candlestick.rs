use crate::trend::Trend;

pub struct Candlestick {
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
}

impl Candlestick {
    pub fn from_candles(candles: &[f64]) -> Self {
        Self {
            open: candles[0],
            close: candles[1],
            high: candles[2],
            low: candles[3],
        }
    }

    pub fn analyze_trend(&self) -> Trend {
        if self.close > self.open {
            Trend::Up
        } else if self.close < self.open {
            Trend::Down
        } else {
            Trend::Unknown
        }
    }

    pub fn has_long_tail(&self) -> bool {
        self.open - self.low > self.high - self.close
    }

    pub fn has_long_head(&self) -> bool {
        self.high - self.close > self.open - self.low
    }
}
