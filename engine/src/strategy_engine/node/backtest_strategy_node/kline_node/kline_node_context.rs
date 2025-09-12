use super::kline_node_type::KlineNodeBacktestConfig;
use crate::strategy_engine::node::node_context::{
    BacktestBaseNodeContext, BacktestNodeContextTrait,
};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use async_trait::async_trait;
use event_center::communication::engine::cache_engine::{CacheEngineResponse, GetCacheParams};
use event_center::communication::engine::exchange_engine::RegisterExchangeParams;
use event_center::communication::engine::market_engine::GetKlineHistoryParams;
use event_center::communication::engine::EngineResponse;
use event_center::communication::strategy::backtest_strategy::command::BacktestStrategyCommand;
use event_center::communication::strategy::backtest_strategy::response::NodeResetResponse;
use event_center::communication::strategy::StrategyCommand;
use event_center::event::node_event::backtest_node_event::kline_node_event::{
    KlineNodeEvent, KlineUpdateEvent, KlineUpdatePayload, TimeUpdateEvent, TimeUpdatePayload,
};
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::SignalEvent;
use event_center::event::Event;
use event_center::EventCenterSingleton;
use heartbeat::Heartbeat;
use star_river_core::cache::key::KlineKey;
use star_river_core::cache::CacheValue;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEvent;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct KlineNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub exchange_is_registered: Arc<RwLock<bool>>,
    pub data_is_loaded: Arc<RwLock<bool>>,
    pub backtest_config: KlineNodeBacktestConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
}

#[async_trait]
impl BacktestNodeContextTrait for KlineNodeContext {
    fn clone_box(&self) -> Box<dyn BacktestNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &BacktestBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BacktestBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        let node_id = self.base_context.node_id.clone();
        self.base_context
            .output_handles
            .get(&format!("{}_default_output", node_id))
            .unwrap()
            .clone()
    }

    async fn handle_event(&mut self, event: Event) {
        let _event = event;
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        // tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, node_event);
        // 收到消息之后，获取对应index的k线数据

        match node_event {
            BacktestNodeEvent::Signal(signal_event) => {
                match signal_event {
                    SignalEvent::KlinePlay(play_event) => {
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
                        for symbol_config in exchange_config.selected_symbols.iter() {
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
                                |handle_id: String, output_handle: NodeOutputHandle| {
                                    let kline_update_event = self.get_kline_update_event(
                                        handle_id,
                                        symbol_config.config_id,
                                        &backtest_kline_key,
                                        current_play_index,
                                        kline_cache_value.clone(),
                                    );
                                    // tracing::debug!("send_kline_event: {:?}", kline_update_event);
                                    let kline_node_event =
                                        BacktestNodeEvent::KlineNode(kline_update_event);

                                    let _ = output_handle.send(kline_node_event);
                                };

                            // 发送到交易对特定的输出handle
                            let symbol_handle_id = symbol_config.output_handle_id.clone();
                            if let Some(symbol_output_handle) =
                                self.base_context.output_handles.get(&symbol_handle_id)
                            {
                                send_kline_event(symbol_handle_id, symbol_output_handle.clone());
                            }

                            // 发送到默认输出handle
                            let default_output_handle = self.get_default_output_handle();
                            send_kline_event(
                                default_output_handle.output_handle_id.clone(),
                                default_output_handle,
                            );

                            // 发送到策略输出handle
                            let strategy_output_handle = self.get_strategy_output_handle();
                            send_kline_event(
                                strategy_output_handle.output_handle_id.clone(),
                                strategy_output_handle.clone(),
                            );
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // 处理策略内部事件
    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) {
        match strategy_inner_event {
            // StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
            //     // 更新k线缓存索引
            //     self.set_play_index(play_index_update_event.play_index).await;
            //     let strategy_output_handle = self.get_strategy_output_handle();
            //     let signal = BacktestNodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
            //         from_node_id: self.get_node_id().clone(),
            //         from_node_name: self.get_node_name().clone(),
            //         from_node_handle_id: strategy_output_handle.output_handle_id.clone(),
            //         play_index: self.get_play_index().await,
            //         message_timestamp: get_utc8_timestamp_millis(),
            //     }));
            //     if let Err(e) = strategy_output_handle.send(signal) {
            //         tracing::error!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "send event failed: {}", e);
            //     }

            // }
            StrategyInnerEvent::NodeReset => {
                // tracing::info!("{}: 收到节点重置事件", self.base_context.node_id);
            }
        }
    }

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) {
        // tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
        match strategy_command {
            StrategyCommand::BacktestStrategy(BacktestStrategyCommand::NodeReset(
                node_reset_params,
            )) => {
                if self.get_node_id() == &node_reset_params.node_id {
                    let response = NodeResetResponse::success(self.get_node_id().clone());
                    node_reset_params.responder.send(response.into()).unwrap();
                }
            }
            _ => {}
        }
    }
}

impl KlineNodeContext {
    // 注册交易所
    #[instrument(skip(self))]
    pub async fn register_exchange(&mut self) -> Result<EngineResponse, String> {
        let account_id = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .account_id
            .clone();
        let exchange = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        let node_id = self.base_context.node_id.clone();
        let node_name = self.base_context.node_name.clone();

        tracing::info!(
            node_id = %node_id,
            node_name = %node_name,
            account_id = %account_id,
            exchange = ?exchange,
            account_id = %account_id,
            "start to register exchange.");

        let (resp_tx, resp_rx) = oneshot::channel();
        let register_param = RegisterExchangeParams::new(account_id, exchange, node_id, resp_tx);

        EventCenterSingleton::send_command(register_param.into())
            .await
            .unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        Ok(response)
    }

    // 从交易所获取k线历史
    #[instrument(skip(self))]
    pub async fn load_kline_history_from_exchange(&mut self) -> Result<bool, String> {
        tracing::info!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "start to load backtest kline data from exchange");
        // 已配置的symbol
        let selected_symbols = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_symbols
            .clone();

        let mut is_all_success = true;
        // 遍历每一个symbol，从交易所获取k线历史
        for symbol in selected_symbols.iter() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let geet_kline_history_params = GetKlineHistoryParams::new(
                self.base_context.strategy_id.clone(),
                self.base_context.node_id.clone(),
                self.backtest_config
                    .exchange_mode_config
                    .as_ref()
                    .unwrap()
                    .selected_account
                    .account_id
                    .clone(),
                self.backtest_config
                    .exchange_mode_config
                    .as_ref()
                    .unwrap()
                    .selected_account
                    .exchange
                    .clone(),
                symbol.symbol.clone(),
                symbol.interval.clone(),
                self.backtest_config
                    .exchange_mode_config
                    .as_ref()
                    .unwrap()
                    .time_range
                    .clone(),
                self.base_context.node_id.clone(),
                resp_tx,
            );
            EventCenterSingleton::send_command(geet_kline_history_params.into())
                .await
                .unwrap();

            let response = resp_rx.await.unwrap();
            if !response.success() {
                is_all_success = false;
                break;
            }
        }
        Ok(is_all_success)
    }

    // 从缓存引擎获取k线数据
    pub async fn get_history_kline_cache(
        &self,
        kline_key: &KlineKey,
        play_index: i32, // 缓存索引
    ) -> Result<Vec<Arc<CacheValue>>, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_cache_params = GetCacheParams::new(
            self.get_strategy_id().clone(),
            self.get_node_id().clone(),
            kline_key.clone().into(),
            Some(play_index as u32),
            Some(1),
            self.get_node_id().clone(),
            resp_tx,
        );

        EventCenterSingleton::send_command(get_cache_params.into())
            .await
            .unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.success() {
            if let Ok(cache_reponse) = CacheEngineResponse::try_from(response) {
                match cache_reponse {
                    CacheEngineResponse::GetCacheData(get_cache_data_response) => {
                        return Ok(get_cache_data_response.cache_data);
                    }
                    _ => {}
                }
            }
        }
        Err(format!("get history kline cache failed"))
    }

    fn get_kline_update_event(
        &self,
        handle_id: String,
        config_id: i32,
        kline_key: &KlineKey,
        index: i32, // 缓存索引
        kline_data: Vec<Arc<CacheValue>>,
    ) -> KlineNodeEvent {
        let payload = KlineUpdatePayload::new(config_id, index, kline_key.clone(), kline_data);
        KlineNodeEvent::KlineUpdate(
            KlineUpdateEvent::new(
                self.get_node_id().clone(),
                self.get_node_name().clone(),
                handle_id,
                payload,
            )
            .into(),
        )
    }
}
