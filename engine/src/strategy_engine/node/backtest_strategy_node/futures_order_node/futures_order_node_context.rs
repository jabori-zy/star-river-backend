use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use types::cache::CacheValue;
use utils::get_utc8_timestamp_millis;
use chrono::Utc;
use event_center::Event;
use uuid::Uuid;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use types::strategy::node_event::BacktestNodeEvent;
use event_center::response::Response;
use event_center::command::Command;
use super::futures_order_node_types::*;
use types::strategy::TradeMode;
use tokio::sync::Mutex;
use crate::Engine;
use crate::exchange_engine::ExchangeEngine;
use std::sync::Arc;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use types::order::Order;
use types::order::OrderStatus;
use database::mutation::order_mutation::OrderMutation;
use tokio::sync::RwLock;
use database::mutation::transaction_mutation::TransactionMutation;
use exchange_client::ExchangeClient;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use types::order::{CreateOrderParams,GetTransactionDetailParams};
use virtual_trading::VirtualTradingSystem;
use types::order::virtual_order::VirtualOrder;
use types::strategy::node_event::SignalEvent;
use tokio::sync::oneshot;
use types::strategy::node_command::{NodeCommand, GetKlineIndexParams, GetStrategyCacheKeysParams};
use types::market::Kline;
use types::cache::key::BacktestKlineKey;
use event_center::command::cache_engine_command::{CacheEngineCommand, GetCacheParams};
use types::market::KlineInterval;
use types::strategy::node_response::NodeResponse;
use types::cache::Key;
use event_center::response::cache_engine_response::CacheEngineResponse;
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use types::order::OrderType;
use types::strategy::node_event::PlayIndexUpdateEvent;
use std::collections::HashMap;
use types::custom_type::OrderId;
use types::strategy::node_event::backtest_node_event::futures_order_node_event::{FuturesOrderNodeEvent, FuturesOrderCreatedEvent, FuturesOrderCanceledEvent, FuturesOrderFilledEvent};
use event_center::command::backtest_strategy_command::StrategyCommand;
use types::virtual_trading_system::event::{VirtualTradingSystemEvent, VirtualTradingSystemEventReceiver};
use types::custom_type::{InputHandleId, NodeId};

#[derive(Debug)]
pub struct FuturesOrderNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: FuturesOrderNodeBacktestConfig,
    pub is_processing_order: Arc<RwLock<HashMap<InputHandleId, bool>>>, // 是否正在处理订单 input_handle_id -> is_processing_order
    pub database: DatabaseConnection, // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>, // 心跳
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>, // 虚拟交易系统
    pub virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver, // 虚拟交易系统事件接收器
    pub unfilled_virtual_order: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualOrder>>>>, // 未成交的虚拟订单列表 input_handle_id -> unfilled_virtual_order
    pub virtual_order_history: Arc<RwLock<HashMap<InputHandleId, Vec<VirtualOrder>>>>, // 虚拟订单历史列表 input_handle_id -> virtual_order_history
    pub min_kline_interval: Option<KlineInterval>, // 最小K线间隔(最新价格只需要获取最小间隔的价格即可)
}

impl Clone for FuturesOrderNodeContext {
    fn clone(&self) -> Self {
        Self {
            base_context: self.base_context.clone(),
            backtest_config: self.backtest_config.clone(),
            is_processing_order: self.is_processing_order.clone(),
            database: self.database.clone(),
            heartbeat: self.heartbeat.clone(),
            virtual_trading_system: self.virtual_trading_system.clone(),
            virtual_trading_system_event_receiver: self.virtual_trading_system_event_receiver.resubscribe(),
            unfilled_virtual_order: self.unfilled_virtual_order.clone(),
            virtual_order_history: self.virtual_order_history.clone(),
            min_kline_interval: self.min_kline_interval.clone(),
        }
    }
}

impl FuturesOrderNodeContext {
    async fn set_is_processing_order(&mut self, input_handle_id: &InputHandleId, is_processing_order: bool) {
        self.is_processing_order.write().await.insert(input_handle_id.to_string(), is_processing_order);
    }

    // 添加未成交的虚拟订单
    async fn add_unfilled_virtual_order(&mut self, input_handle_id: &InputHandleId, virtual_order: VirtualOrder) {
        let mut unfilled_virtual_order_guard = self.unfilled_virtual_order.write().await;
        tracing::info!("{}: 订单已添加到未成交的虚拟订单列表: {:?}", self.get_node_id(), virtual_order);
        unfilled_virtual_order_guard.entry(input_handle_id.to_string()).or_insert(vec![]).push(virtual_order);
        
    }

    async fn remove_unfilled_virtual_order(&mut self, input_handle_id: &InputHandleId, virtual_order_id: OrderId) {
        let mut unfilled_virtual_order_guard = self.unfilled_virtual_order.write().await;
        unfilled_virtual_order_guard.entry(input_handle_id.to_string()).and_modify(|orders| {
            orders.retain(|order| order.order_id != virtual_order_id);
        });
    }

    async fn add_virtual_order_history(&mut self, input_handle_id: &InputHandleId, virtual_order: VirtualOrder) {
        let mut virtual_order_history_guard = self.virtual_order_history.write().await;
        virtual_order_history_guard.entry(input_handle_id.to_string()).or_insert(vec![]).push(virtual_order);
    }

    async fn remove_virtual_order_history(&mut self, input_handle_id: &InputHandleId, virtual_order_id: OrderId) {
        let mut virtual_order_history_guard = self.virtual_order_history.write().await;
        virtual_order_history_guard.entry(input_handle_id.to_string()).and_modify(|orders| {
            orders.retain(|order| order.order_id != virtual_order_id);
        });
    }

    // 判断是否可以创建订单
    async fn can_create_order(&mut self, input_handle_id: &InputHandleId) -> bool {
        let is_processing_order_guard = self.is_processing_order.read().await;
        let is_processing_order = *is_processing_order_guard.get(input_handle_id).unwrap_or(&false);
        
        let unfilled_virtual_order_guard = self.unfilled_virtual_order.read().await;
        let unfilled_virtual_order_len = unfilled_virtual_order_guard
            .get(input_handle_id)
            .map_or(0, |v| v.len());

        !(is_processing_order || unfilled_virtual_order_len > 0)
    }

    async fn create_order(&mut self, order_config: &FuturesOrderConfig) -> Result<(), String> {
        // 如果当前是正在处理订单的状态，或者未成交的订单列表不为空，则不创建订单
        if !self.can_create_order(&order_config.input_handle_id).await {
            tracing::warn!("{}: 当前正在处理订单, 跳过", self.get_node_name());
            return Err("当前正在处理订单, 跳过".to_string());
        }

        // 将input_handle_id的is_processing_order设置为true
        self.set_is_processing_order(&order_config.input_handle_id, true).await;
        tracing::info!("{}: 开始创建订单", self.get_node_id());

        let mut virtual_trading_system_guard = self.virtual_trading_system.lock().await;
        let exchange = self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.exchange.clone();
        // 创建订单
        virtual_trading_system_guard.create_order(
            self.get_strategy_id().clone(),
            self.get_node_id().clone(),
            order_config.order_config_id,
            order_config.symbol.clone(),
            exchange,
            order_config.price,
            order_config.order_side.clone(),
            order_config.order_type.clone(),
            order_config.quantity,
            order_config.tp,
            order_config.sl,
        )?;

        // // 释放virtual_trading_system_guard
        // drop(virtual_trading_system_guard);
        
        // 重置is_processing_order
        // self.set_is_processing_order(order_config.input_handle_id.as_str(), false).await;
        Ok(())
    }

    pub async fn handle_node_event_for_specific_order(
        &mut self, 
        node_event: BacktestNodeEvent, 
        from_node_id: &NodeId, 
        input_handle_id: &InputHandleId
    ) -> Result<(), String> {
        tracing::debug!("{}: 接收器 {} 接收到节点事件: {:?} 来自节点: {}", self.get_node_id(), input_handle_id, node_event, from_node_id);
        match node_event {
            BacktestNodeEvent::Signal(signal_event) => {
                match signal_event {
                    SignalEvent::BacktestConditionMatch(backtest_condition_match_event) => {
                        if backtest_condition_match_event.play_index == self.get_play_index().await {
                            // 根据input_handle_id获取订单配置
                            let order_config = {
                                self.backtest_config.futures_order_configs
                                .iter()
                                .find(|config| config.input_handle_id == *input_handle_id)
                                .ok_or("订单配置不存在".to_string())?
                                .clone()
                            };
                            // 创建订单
                            self.create_order(&order_config).await?;
                        }
                        else {
                            tracing::warn!("{}: 当前k线缓存索引不匹配, 跳过", self.get_node_id());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())

    }


    async fn check_order_status(&mut self, order_id: OrderId) -> Result<OrderStatus, String> {
        let virtual_trading_system_guard = self.virtual_trading_system.lock().await;
        let order = virtual_trading_system_guard.get_order(order_id);
        if let Some(order) = order {
            return Ok(order.order_status.clone());
        }
        Err("订单不存在".to_string())
    }


    async fn send_order_status_event(&mut self, virtual_order: VirtualOrder) {
        // 订单状态
        let order_status = &virtual_order.order_status;

        let output_handle_id = format!("{}_{}_output{}", self.get_node_id(), order_status.to_string(), virtual_order.order_config_id);
        tracing::debug!("{}: 获取输出句柄: {:?}", self.get_node_id(), output_handle_id);

        let output_handle = self.get_output_handle(&output_handle_id);
        tracing::debug!("{}: 发送订单状态事件: {:?}", self.get_node_id(), output_handle);
        let order_event = match order_status {
            OrderStatus::Filled => FuturesOrderNodeEvent::FuturesOrderFilled(FuturesOrderFilledEvent {
                from_node_id: self.get_node_id().clone(),
                from_node_name: self.get_node_name().clone(),
                from_handle_id: output_handle_id.clone(),
                futures_order: virtual_order.clone(),
                timestamp: get_utc8_timestamp_millis(),
            }),
            OrderStatus::Created => FuturesOrderNodeEvent::FuturesOrderCreated(FuturesOrderCreatedEvent {
                from_node_id: self.get_node_id().clone(),
                from_node_name: self.get_node_name().clone(),
                from_handle_id: output_handle_id.clone(),
                futures_order: virtual_order.clone(),
                timestamp: get_utc8_timestamp_millis(),
            }),
            OrderStatus::Canceled => FuturesOrderNodeEvent::FuturesOrderCanceled(FuturesOrderCanceledEvent {
                from_node_id: self.get_node_id().clone(),
                from_node_name: self.get_node_name().clone(),
                from_handle_id: output_handle_id.clone(),
                futures_order: virtual_order.clone(),
                timestamp: get_utc8_timestamp_millis(),
            }),
            _ => return,
        };
        if let Err(e) = output_handle.send(order_event.clone().into()) {
            tracing::error!("{}: 发送订单状态事件失败: {:?}", self.get_node_id(), e);
        }

        // 给策略发订单事件
        let strategy_output_handle = self.get_strategy_output_handle();
        let _ = strategy_output_handle.send(order_event.into());


        
        
    }

    // async fn send_test_signal(&mut self) {
    //     let output_handle = self.get_all_output_handle().get("order_node_output").unwrap();
    //     let order_message = OrderMessage::OrderFilled(Order {
    //         order_id: 1,
    //         strategy_id: self.get_strategy_id().clone(),
    //         node_id: self.get_node_id().clone(),
    //         exchange_order_id: 475265246,
    //         account_id: self.backtest_config.selected_backtest_accounts[0],
    //         exchange: self.backtest_config.order_config.exchange.clone(),
    //         symbol: self.backtest_config.order_config.symbol.clone(),
    //         order_side: self.backtest_config.order_config.order_side.clone(),
    //         order_type: self.backtest_config.order_config.order_type.clone(),
    //         order_status: OrderStatus::Filled,
    //         quantity: self.backtest_config.order_config.quantity,
    //         open_price: self.backtest_config.order_config.price,
    //         tp: self.backtest_config.order_config.tp,
    //         sl: self.backtest_config.order_config.sl,
    //         extra_info: Some(serde_json::to_value("111".to_string()).unwrap()),
    //         created_time: Utc::now(),
    //         updated_time: Utc::now(),
    //     });
    //     output_handle.send(NodeMessage::Order(order_message.clone())).unwrap();
    // }

    pub async fn monitor_unfilled_order(&mut self) {
        let unfilled_virtual_order = self.unfilled_virtual_order.clone();
        let is_processing_order = self.is_processing_order.clone();
        let database = self.database.clone();
        let node_name = self.get_node_name().clone();

        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(
            format!("{}监控未成交订单", self.get_node_name()),
            move || {
                let unfilled_virtual_order = unfilled_virtual_order.clone();
                let is_processing_order = is_processing_order.clone();
                let database = database.clone();
                let node_name = node_name.clone();
                async move {
                    Self::process_unfilled_virtual_order(
                        node_name,
                        unfilled_virtual_order,
                        is_processing_order,
                        database
                    ).await
                }
            },
            10
        ).await;
    }

    async fn process_unfilled_virtual_order(
        node_name: String,
        unfilled_virtual_order: Arc<RwLock<HashMap<String, Vec<VirtualOrder>>>>,
        is_processing_order: Arc<RwLock<HashMap<String, bool>>>,
        database: DatabaseConnection,
    ) {
        let unfilled_virtual_order_clone = {
            let unfilled_order_guard = unfilled_virtual_order.read().await;
            unfilled_order_guard.clone()
        };
        // 如果当前没有正在处理的订单，则直接返回
        if unfilled_virtual_order_clone.len() == 0 {
            // tracing::info!("{}: 没有未成交订单", node_name);
            return;
        }
        // 如果当前有正在处理的订单，则获取订单的最新状态


    }

    async fn get_strategy_cache_keys(&mut self) -> Result<Vec<Key>, String> {
        let (tx, rx) = oneshot::channel();
        let node_command = NodeCommand::GetStrategyCacheKeys(GetStrategyCacheKeysParams {
            node_id: self.get_node_id().clone(),
            timestamp: Utc::now().timestamp_millis(),
            responder: tx,
        });

        self.get_node_command_sender().send(node_command).await.unwrap();

        let response = rx.await.unwrap();
        match response {
            NodeResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                return Ok(get_strategy_cache_keys_response.cache_keys)
            }
        }
    }

    // 获取K线缓存数据
    // 获取interval最小的K线缓存数据
    async fn get_kline_cache_data(&mut self) -> Result<Vec<Arc<CacheValue>>, String> {
        // 如果min_kline_interval为None，则获取策略的缓存键
        if self.min_kline_interval.is_none() {
            let cache_keys = self.get_strategy_cache_keys().await;
            // 获取成功
            if let Ok(cache_keys) = cache_keys {
                // 过滤出K线缓存key
                let kline_cache_keys = cache_keys.iter().filter(|k| matches!(k, Key::BacktestKline(_))).collect::<Vec<&Key>>();
                // 获取interval最小的K线缓存数据
                // 如果列表长度为1，则唯一的key就是最小interval的key
                if kline_cache_keys.len() == 1 {
                    self.min_kline_interval = Some(kline_cache_keys[0].get_interval());
                } else if !kline_cache_keys.is_empty() {
                    // 如果列表长度大于1，则需要根据interval排序，获取最小的interval的key
                    let min_interval_key = kline_cache_keys.iter()
                        .min_by_key(|k| k.get_interval())
                        .unwrap(); // 这里可以安全unwrap，因为我们已经检查了不为空
                    self.min_kline_interval = Some(min_interval_key.get_interval());
                }
            }
        }
        // 如果min_kline_interval不为None，则获取K线缓存数据
        if let Some(min_kline_interval) = &self.min_kline_interval {
            let cache_key = BacktestKlineKey::new(
                self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.exchange.clone(),
                self.backtest_config.futures_order_configs[0].symbol.clone(),
                min_kline_interval.clone(),
                self.backtest_config.exchange_mode_config.as_ref().unwrap().time_range.start_date.to_string(),
                self.backtest_config.exchange_mode_config.as_ref().unwrap().time_range.end_date.to_string(),
            );


            let play_index = self.get_play_index().await as u32;

            let (tx, rx) = oneshot::channel();
            let get_cache_params = GetCacheParams {
                strategy_id: self.base_context.strategy_id.clone(),
                node_id: self.base_context.node_id.clone(),
                key: cache_key.clone().into(),
                index: Some(play_index),
                limit: Some(1),
                sender: self.base_context.node_id.clone(),
                timestamp: get_utc8_timestamp_millis(),
                responder: tx,
            };
            let get_cache_command = CacheEngineCommand::GetCache(get_cache_params);

            self.get_command_publisher().send(get_cache_command.into()).await.unwrap();

            let reponse = rx.await.unwrap();
            match reponse {
                Response::CacheEngine(CacheEngineResponse::GetCacheData(get_cache_data_response)) => {
                    // tracing::info!("{}: 获取K线缓存数据成功: {:?}", self.get_node_id(), get_cache_data_response.cache_data);
                    return Ok(get_cache_data_response.cache_data)
                }
                _ => return Err("获取K线缓存数据失败".to_string()),
            }
        } else {
            return Err("获取K线缓存数据失败".to_string());
        }

        
    }

    // 处理虚拟交易系统事件
    pub async fn handle_virtual_trading_system_event(&mut self, virtual_trading_system_event: VirtualTradingSystemEvent) -> Result<(), String> {
        match virtual_trading_system_event {
            VirtualTradingSystemEvent::FuturesOrderCreated(order) => {
                // 判断是否是本节点创建的订单
                if order.node_id == self.get_node_id().clone() {
                    // 说明是本节点创建的订单
                    // 获取input_handle_id
                    let input_handle_id = format!("{}_input{}", self.get_node_id(), order.order_config_id);
                    // 将订单放到未成交的虚拟订单列表中
                    self.add_unfilled_virtual_order(&input_handle_id, order.clone()).await;
                    // 这里不设置is_processing_order为false，必须等待订单完全成交后，才能设置为false

                    // 发送订单事件
                    self.send_order_status_event(order).await;
                }
            }
            VirtualTradingSystemEvent::FuturesOrderFilled(order) => {
                if order.node_id == self.get_node_id().clone() {
                    // 说明是本节点创建的订单
                    // 获取input_handle_id
                    let input_handle_id = format!("{}_input{}", self.get_node_id(), order.order_config_id);
                    // 将订单从未成交的虚拟订单列表中移除
                    self.remove_unfilled_virtual_order(&input_handle_id, order.order_id).await;
                    // 将订单放到虚拟订单历史列表中
                    self.add_virtual_order_history(&input_handle_id, order.clone()).await;
                    // 将is_processing_order设置为false
                    self.set_is_processing_order(&input_handle_id, false).await;
                    // 发送订单事件
                    self.send_order_status_event(order).await;
                }
            }
            _ => {}
        }
        Ok(())
    }

}

#[async_trait]
impl BacktestNodeContextTrait for FuturesOrderNodeContext {
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
        self.base_context.output_handles.get(&format!("order_node_output")).unwrap().clone()
    }
    
    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        // match event {
        //     Event::Response(response_event) => {
        //         self.handle_response_event(response_event).await;
        //     }
        //     _ => {}
        // }
        Ok(())
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) -> Result<(), String> {
        tracing::debug!("{}: 接收到节点事件: {:?}", self.get_node_id(), node_event);
        // match node_event {
        //     NodeEvent::Signal(signal_event) => {
        //         match signal_event {
        //             SignalEvent::BacktestConditionMatch(backtest_condition_match_event) => {
        //                 if backtest_condition_match_event.play_index == self.get_play_index().await {
        //                     self.create_order().await;
        //                 }
        //                 else {
        //                     tracing::warn!("{}: 当前k线缓存索引不匹配, 跳过", self.get_node_id());
        //                 }
        //             }
        //             _ => {}
        //         }
        //     }
        //     _ => {}
        // }
        Ok(())
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) -> Result<(), String> {
        match strategy_inner_event {
            StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
                // 更新k线缓存索引
                self.set_play_index(play_index_update_event.play_index).await;
                let strategy_output_handle_id = format!("{}_strategy_output", self.get_node_id());
                let signal = BacktestNodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
                    from_node_id: self.get_node_id().clone(),
                    from_node_name: self.get_node_name().clone(),
                    from_node_handle_id: strategy_output_handle_id.clone(),
                    play_index: self.get_play_index().await,
                    message_timestamp: get_utc8_timestamp_millis(),
                }));
                self.get_strategy_output_handle().send(signal).unwrap();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) -> Result<(), String> {
        tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
        Ok(())
    }

}


