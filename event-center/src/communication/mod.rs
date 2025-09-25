pub mod engine;
pub mod backtest_strategy;


use star_river_core::system::DateTimeUtc;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use std::sync::Arc;

pub trait Command {
    type Response;

    fn datetime(&self) -> DateTimeUtc;
    fn respond(self, response: Self::Response);
}


pub trait Response {
    fn is_success(&self) -> bool;
    fn get_error(&self) -> Arc<dyn StarRiverErrorTrait + Send + Sync>;
    fn datetime(&self) -> DateTimeUtc;
}