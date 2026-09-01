//! Grind mode: an offline typing trainer for classic leetcode solutions with
//! a live candlestick chart of your words-per-minute, rendered like a stock
//! ticker. Launch with `leetui grind`.

pub mod app;
pub mod candles;
pub mod chart;
pub mod engine;
pub mod problems;
pub mod theme;
pub mod ui;

pub use app::run;
