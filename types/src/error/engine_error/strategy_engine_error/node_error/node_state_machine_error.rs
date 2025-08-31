


use snafu::{Snafu, Backtrace};





#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestNodeStateMachineError {

    #[snafu(display("fail to transition from {from_state} to {to_state}, event: {event}"))]
    NodeTransition {
        from_state: String,
        to_state: String,
        event: String,
        backtrace: Backtrace,
    }

    
}