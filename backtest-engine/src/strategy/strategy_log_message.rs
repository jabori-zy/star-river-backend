use serde::{Deserialize, Serialize};
use strategy_core::{log_message, log_message::*};
log_message!(
    StrategyStateLogMsg,
    params: (
        strategy_name: String,
        strategy_state: String,
    ),
    en: "#[{strategy_name}] current state is: {strategy_state}",
    zh: "#[{strategy_name}] 当前状态是：{strategy_state}"
);
