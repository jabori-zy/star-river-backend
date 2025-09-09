use crate::log_message;
use crate::strategy_engine::log_message::*;
use serde::{Deserialize, Serialize};

log_message!(
    StrategyStateLogMsg,
    params: (
        strategy_id: i32,
        strategy_name: String,
        strategy_state: String,
    ),
    en: "Strategy [{strategy_name}({strategy_id})] current state is: {strategy_state}",
    zh: "{strategy_name} ({strategy_id}) 当前状态是: {strategy_state}"
);
