use thiserror::Error;

#[derive(Error, Debug)]
pub enum TalibError {
    #[error("Failed to calculate SMA: period= {period}, error= {error}")]
    CalculateSMAError { period: i32, error: String },
}
