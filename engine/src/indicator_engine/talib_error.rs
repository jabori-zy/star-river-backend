use thiserror::Error;

#[derive(Error, Debug)]
pub enum TalibError {
    #[error("Failed to calculate SMA: period= {period}, error= {error}")]
    CalculateSMAError { period: i32, error: String },
    #[error("Failed to calculate MACD: fast_period= {fast_period}, slow_period= {slow_period}, signal_period= {signal_period}, error= {error}")]
    CalculateMACDError { fast_period: i32, slow_period: i32, signal_period: i32, error: String },
    #[error("Failed to calculate BBANDS: period= {period}, dev_up= {dev_up}, dev_down= {dev_down}, ma_type= {ma_type}, error= {error}")]
    CalculateBBANDSError { period: i32, dev_up: f64, dev_down: f64, ma_type: i32, error: String },
}
