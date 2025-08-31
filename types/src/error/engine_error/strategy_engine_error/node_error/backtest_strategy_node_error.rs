pub mod kline_node_error;


use snafu::{Snafu, Backtrace};


pub use kline_node_error::KlineNodeError;
use super::node_state_machine_error::BacktestNodeStateMachineError;




#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestStrategyNodeError {

    #[snafu(transparent)]
    StateMachine {
        source: BacktestNodeStateMachineError,
        backtrace: Backtrace,
    },


    #[snafu(transparent)]
    KlineNode {
        source: KlineNodeError,
        backtrace: Backtrace,
    },
}