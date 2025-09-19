use super::KlineNodeContext;
use crate::strategy_engine::node::node_context::BacktestNodeContextTrait;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use event_center::event::node_event::backtest_node_event::kline_node_event::{
    KlineNodeEvent, TimeUpdateEvent, TimeUpdatePayload,
};
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::CommonEvent;
use star_river_core::cache::key::KlineKey;
use event_center::event::node_event::backtest_node_event::start_node_event::KlinePlayEvent;
use event_center::event::node_event::backtest_node_event::common_event::{ExecuteOverPayload, ExecuteOverEvent};
use star_river_core::cache::Key;
use event_center::communication::strategy::NodeResponse;
use event_center::communication::strategy::backtest_strategy::command::GetStrategyKeysParams;
use event_center::communication::strategy::backtest_strategy::response::BacktestNodeResponse;
use tokio::sync::oneshot;


impl KlineNodeContext {
    pub(super) async fn send_kline(&self, play_event: KlinePlayEvent) {
        // 提前获取配置信息，统一错误处理
        let exchange_config =
        self.backtest_config.exchange_mode_config.as_ref().unwrap();

    // let current_play_index = self.get_play_index().await;
    // tracing::debug!("current_play_index: {}", current_play_index);

    let current_play_index = self.get_play_index();

    // 如果索引不匹配，提前返回错误日志
    if current_play_index != play_event.play_index {
        tracing::error!(
            node_id = %self.base_context.node_id,
            node_name = %self.base_context.node_name,
            kline_cache_index = %play_event.play_index,
            signal_index = %current_play_index,
            "kline cache index is not equal to signal index"
        );
        return;
    }

    // 提取公共数据

    let exchange = exchange_config.selected_account.exchange.clone();
    let start_time = exchange_config.time_range.start_date.to_string();
    let end_time = exchange_config.time_range.end_date.to_string();

    // 循环处理所有选定的交易对
    let mut pre_kline_timestamp = 0;
    for (index, symbol_config) in exchange_config.selected_symbols.iter().enumerate() {
        // 创建k线缓存键
        let backtest_kline_key = KlineKey::new(
            exchange.clone(),
            symbol_config.symbol.clone(),
            symbol_config.interval.clone(),
            Some(start_time.clone()),
            Some(end_time.clone()),
        );

        // 获取k线缓存值
        let kline_cache_value = self
            .get_history_kline_cache(&backtest_kline_key, current_play_index)
            .await;
        let kline_cache_value = match kline_cache_value {
            Ok(value) => value,
            Err(e) => {
                tracing::error!(
                    node_id = %self.base_context.node_id,
                    node_name = %self.base_context.node_name,
                    symbol = %symbol_config.symbol,
                    "Failed to get history kline cache: {}", e
                );
                continue;
            }
        };

        let kline_timestamp = kline_cache_value.last().unwrap().get_timestamp();
        // 如果时间戳不等于上一根k线的时间戳，并且上一根k线的时间戳为0， 初始值，则发送时间更新事件
        if pre_kline_timestamp != kline_timestamp && pre_kline_timestamp == 0 {
            pre_kline_timestamp = kline_timestamp;
            let kline_datetime = kline_cache_value.last().unwrap().get_datetime();
            let payload = TimeUpdatePayload::new(kline_datetime);
            let time_update_event: KlineNodeEvent = TimeUpdateEvent::new(
                self.get_node_id().clone(),
                self.get_node_name().clone(),
                self.get_node_id().clone(),
                payload,
            )
            .into();
            self.get_strategy_output_handle()
                .send(time_update_event.into())
                .unwrap();
        }
        // 如果时间戳不等于上一根k线的时间戳，并且上一根k线的时间戳不为0，说明有错误，同一批k线的时间戳不一致
        else if pre_kline_timestamp != kline_timestamp
            && pre_kline_timestamp != 0
        {
            tracing::error!(
                node_id = %self.base_context.node_id,
                node_name = %self.base_context.node_name,
                symbol = %symbol_config.symbol,
                "kline timestamp is not equal to previous kline timestamp"
            );
            continue;
        }

        // 发送K线更新事件的通用函数

        let send_kline_event =
            |handle_id: String, output_handle: &NodeOutputHandle| {
                let kline_update_event = self.get_kline_update_event(
                    handle_id,
                    symbol_config.config_id,
                    &backtest_kline_key,
                    current_play_index,
                    kline_cache_value.clone(),
                );
                // tracing::debug!("send_kline_event: {:?}", kline_update_event);
                let kline_node_event = BacktestNodeEvent::KlineNode(kline_update_event);

                let _ = output_handle.send(kline_node_event);
            };

        // 发送到交易对特定的输出handle
        let symbol_handle_id = symbol_config.output_handle_id.clone();
        let symbol_ouput_handle = self.get_output_handle(&symbol_handle_id);
        if symbol_ouput_handle.connect_count > 0 {
            send_kline_event(symbol_handle_id, symbol_ouput_handle);
        }

        // 发送到默认输出handle
        let default_output_handle = self.get_default_output_handle();
        if default_output_handle.connect_count > 0 {
            send_kline_event(default_output_handle.output_handle_id.clone(), default_output_handle);
        }
        

        // 发送到策略输出handle
        let strategy_output_handle = self.get_strategy_output_handle();
        send_kline_event(
            strategy_output_handle.output_handle_id.clone(),
            strategy_output_handle,
        );

        if index == exchange_config.selected_symbols.len() - 1 {
            if self.is_leaf_node() {
                self.send_execute_over_event().await;
            }
        }



        
    }
    }



    pub async fn get_strategy_keys(&mut self) -> Result<Vec<Key>, String> {
        let (tx, rx) = oneshot::channel();
        let get_strategy_keys_params = GetStrategyKeysParams::new(self.get_node_id().clone(), tx);

        self.get_node_command_sender()
            .send(get_strategy_keys_params.into())
            .await
            .unwrap();

        let response = rx.await.unwrap();
        match response {
            NodeResponse::BacktestNode(BacktestNodeResponse::GetStrategyCacheKeys(
                get_strategy_keys_response,
            )) => return Ok(get_strategy_keys_response.keys),
            _ => return Err("获取策略缓存键失败".to_string()),
        }
    }

}