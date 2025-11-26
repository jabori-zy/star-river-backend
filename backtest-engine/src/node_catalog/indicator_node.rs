mod context;
mod indicator_node_type;
mod node_lifecycle;
mod state_machine;

// Standard library imports
use std::{collections::HashMap, str::FromStr, sync::Arc};

use chrono::{DateTime, Utc};
// Local module imports
use context::IndicatorNodeContext;
use indicator_node_type::{ExchangeModeConfig, IndicatorNodeBacktestConfig};
// External project crates
use key::{IndicatorKey, KlineKey};
// External crate imports
use serde::de::IntoDeserializer;
use snafu::ResultExt;
use star_river_core::{
    custom_type::{NodeId, NodeName, StrategyId},
    system::deserialize_time_range,
};
use state_machine::{IndicatorNodeStateMachine, indicator_node_transition};
use strategy_core::{
    NodeType,
    error::node_error::{ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu},
    node::{NodeBase, metadata::NodeMetadata, node_trait::NodeContextAccessor, utils::generate_strategy_output_handle},
    strategy::{SelectedAccount, SelectedIndicator, SelectedSymbol, cycle::Cycle},
};
use ta_lib::IndicatorConfig;
use tokio::sync::{Mutex, RwLock, mpsc, watch};

// Crate imports
use crate::{
    node::{
        node_command::BacktestNodeCommand,
        node_error::{IndicatorNodeError, indicator_node_error::DataSourceParseFailedSnafu},
        node_event::BacktestNodeEvent,
        node_state_machine::NodeRunState,
    },
    strategy::{strategy_command::BacktestStrategyCommand, strategy_config::BacktestDataSource},
};

#[derive(Debug, Clone)]
pub struct IndicatorNode {
    inner: NodeBase<IndicatorNodeContext>,
}

impl std::ops::Deref for IndicatorNode {
    type Target = NodeBase<IndicatorNodeContext>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NodeContextAccessor for IndicatorNode {
    type Context = IndicatorNodeContext;
    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        self.inner.context()
    }
}

impl IndicatorNode {
    pub fn new(
        cycle_rx: watch::Receiver<Cycle>,
        node_config: serde_json::Value,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<BacktestNodeCommand>>>,
        strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>,
    ) -> Result<Self, IndicatorNodeError> {
        let (strategy_id, node_id, node_name, node_config) = Self::check_indicator_node_config(node_config)?;

        let strategy_bound_handle = generate_strategy_output_handle::<BacktestNodeEvent>(&node_id, &node_name);

        let state_machine = IndicatorNodeStateMachine::new(node_name.clone(), NodeRunState::Created, indicator_node_transition);

        let metadata = NodeMetadata::new(
            cycle_rx,
            strategy_time_watch_rx,
            strategy_id,
            node_id,
            node_name,
            NodeType::IndicatorNode,
            state_machine,
            strategy_bound_handle,
            strategy_command_sender,
            node_command_receiver,
        );
        // 通过配置，获取指标缓存键
        let indicator_keys = Self::get_indicator_keys(&node_config)?;
        // 通过配置，获取回测K线缓存键
        let selected_kline_key = Self::get_kline_key(&node_config)?;

        let context = IndicatorNodeContext::new(metadata, node_config, selected_kline_key, indicator_keys);
        Ok(Self {
            inner: NodeBase::new(context),
        })
    }

    fn check_indicator_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, IndicatorNodeBacktestConfig), IndicatorNodeError> {
        let node_id = node_config
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "id".to_string(),
                }
                .build()
            })?
            .to_owned();
        let node_data = node_config
            .get("data")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "data".to_string(),
                }
                .build()
            })?
            .to_owned();
        let node_name = node_data
            .get("nodeName")
            .and_then(|name| name.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "nodeName".to_string(),
                }
                .build()
            })?
            .to_owned();
        let strategy_id = node_data
            .get("strategyId")
            .and_then(|id| id.as_i64())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "strategyId".to_string(),
                }
                .build()
            })?
            .to_owned() as StrategyId;

        let backtest_config_json = node_data
            .get("backtestConfig")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "backtestConfig".to_string(),
                }
                .build()
            })?
            .to_owned();

        let selected_account_json = backtest_config_json
            .get("exchangeModeConfig")
            .and_then(|config| config.get("selectedAccount"))
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "selectedAccount".to_string(),
                }
                .build()
            })?
            .to_owned();
        let selected_account =
            serde_json::from_value::<SelectedAccount>(selected_account_json).context(ConfigDeserializationFailedSnafu {
                node_name: node_name.clone(),
            })?;

        let selected_symbol_json = backtest_config_json
            .get("exchangeModeConfig")
            .and_then(|config| config.get("selectedSymbol"))
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "selectedSymbol".to_string(),
                }
                .build()
            })?
            .to_owned();
        let selected_symbol = serde_json::from_value::<SelectedSymbol>(selected_symbol_json).context(ConfigDeserializationFailedSnafu {
            node_name: node_name.clone(),
        })?;

        let time_range_json = backtest_config_json
            .get("exchangeModeConfig")
            .and_then(|config| config.get("timeRange"))
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "timeRange".to_string(),
                }
                .build()
            })?
            .to_owned();
        let time_range = deserialize_time_range(time_range_json.into_deserializer()).context(ConfigDeserializationFailedSnafu {
            node_name: node_name.clone(),
        })?;

        let data_source = backtest_config_json
            .get("dataSource")
            .and_then(|source| source.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "dataSource".to_string(),
                }
                .build()
            })?
            .to_owned();
        let data_source = BacktestDataSource::from_str(&data_source).context(DataSourceParseFailedSnafu { data_source })?;

        let selected_indicators_array = backtest_config_json
            .get("exchangeModeConfig")
            .and_then(|config| config.get("selectedIndicators"))
            .and_then(|indicators| indicators.as_array())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "selectedIndicators".to_string(),
                }
                .build()
            })?
            .to_owned();

        let mut selected_indicators = Vec::new();
        for ind_config in selected_indicators_array {
            let indicator_type = ind_config.get("indicatorType").and_then(|t| t.as_str()).ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "indicatorType".to_string(),
                }
                .build()
            })?;
            let indicator_config_json = ind_config
                .get("indicatorConfig")
                .ok_or_else(|| {
                    ConfigFieldValueNullSnafu {
                        field_name: "indicatorConfig".to_string(),
                    }
                    .build()
                })?
                .to_owned();
            let indicator_config = IndicatorConfig::new(indicator_type, &indicator_config_json).map_err(IndicatorNodeError::from)?;
            let config_id = ind_config
                .get("configId")
                .and_then(|id| id.as_i64())
                .ok_or_else(|| {
                    ConfigFieldValueNullSnafu {
                        field_name: "configId".to_string(),
                    }
                    .build()
                })?
                .to_owned() as i32;
            let output_handle_id = ind_config
                .get("outputHandleId")
                .and_then(|id| id.as_str())
                .ok_or_else(|| {
                    ConfigFieldValueNullSnafu {
                        field_name: "outputHandleId".to_string(),
                    }
                    .build()
                })?
                .to_owned();
            selected_indicators.push(SelectedIndicator {
                config_id,
                output_handle_id,
                indicator_config,
            });
        }
        let exchange_mode_config = ExchangeModeConfig {
            selected_account,
            selected_symbol,
            selected_indicators,
            time_range,
        };

        let backtest_config = IndicatorNodeBacktestConfig {
            node_name: node_name.clone(),
            data_source,
            exchange_mode_config: Some(exchange_mode_config),
            file_mode_config: None,
        };
        Ok((strategy_id, node_id, node_name, backtest_config))
    }

    fn get_indicator_keys(
        backtest_config: &IndicatorNodeBacktestConfig,
    ) -> Result<HashMap<IndicatorKey, (i32, String)>, IndicatorNodeError> {
        let exchange = backtest_config.exchange_mode()?.selected_account.exchange.clone();
        let symbol = backtest_config.exchange_mode()?.selected_symbol.symbol.clone();
        let interval = backtest_config.exchange_mode()?.selected_symbol.interval.clone();
        let time_range = backtest_config.exchange_mode()?.time_range.clone();

        let mut indicator_keys = HashMap::new();
        for indicator in backtest_config.exchange_mode()?.selected_indicators.iter() {
            let indicator_key = IndicatorKey {
                exchange: exchange.clone(),
                symbol: symbol.clone(),
                interval: interval.clone(),
                indicator_config: indicator.indicator_config.clone(),
                start_time: Some(time_range.start_date.to_string()),
                end_time: Some(time_range.end_date.to_string()),
            };
            indicator_keys.insert(indicator_key, (indicator.config_id, indicator.output_handle_id.clone()));
        }
        Ok(indicator_keys)
    }

    fn get_kline_key(backtest_config: &IndicatorNodeBacktestConfig) -> Result<KlineKey, IndicatorNodeError> {
        let exchange = backtest_config.exchange_mode()?.selected_account.exchange.clone();
        let symbol = backtest_config.exchange_mode()?.selected_symbol.symbol.clone();
        let interval = backtest_config.exchange_mode()?.selected_symbol.interval.clone();
        let time_range = backtest_config.exchange_mode()?.time_range.clone();

        let kline_key = KlineKey {
            exchange: exchange,
            symbol: symbol,
            interval: interval,
            start_time: Some(time_range.start_date.to_string()),
            end_time: Some(time_range.end_date.to_string()),
        };
        Ok(kline_key)
    }
}
