use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use types::cache::CacheValue;
use utils::get_utc8_timestamp_millis;
use chrono::Utc;
use event_center::Event;
use uuid::Uuid;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use types::strategy::node_event::NodeEvent;
use event_center::response::Response;
use event_center::command::Command;
use super::order_node_types::*;
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
use types::cache::cache_key::BacktestKlineCacheKey;
use event_center::command::cache_engine_command::{CacheEngineCommand, GetCacheParams};
use types::market::KlineInterval;
use types::strategy::node_response::NodeResponse;
use types::cache::CacheKey;
use event_center::response::cache_engine_response::CacheEngineResponse;
use types::strategy::strategy_inner_event::StrategyInnerEvent;

#[derive(Debug, Clone)]
pub struct OrderNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: OrderNodeBacktestConfig,
    pub is_processing_order: Arc<RwLock<bool>>, // 是否正在处理订单
    pub database: DatabaseConnection, // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>, // 心跳
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>, // 虚拟交易系统
    pub unfilled_virtual_order: Arc<RwLock<Vec<VirtualOrder>>>, // 未成交的虚拟订单列表
    pub min_kline_interval: Option<KlineInterval>, // 最小K线间隔(最新价格只需要获取最小间隔的价格即可)
    pub kline_cache_index: Arc<RwLock<u32>>, // 回测播放索引
}

impl OrderNodeContext {
    async fn set_is_processing_order(&mut self, is_processing_order: bool) {
        *self.is_processing_order.write().await = is_processing_order;
    }

    // 添加未成交的虚拟订单
    async fn add_unfilled_virtual_order(&mut self, virtual_order: VirtualOrder) {
        self.unfilled_virtual_order.write().await.push(virtual_order);
    }

    async fn create_order(&mut self) {
        // 如果当前是正在处理订单的状态，或者未成交的虚拟订单列表不为空，则不创建订单
        if *self.is_processing_order.read().await || self.unfilled_virtual_order.read().await.len() > 0 {
            // tracing::warn!("{}: 当前正在处理订单, 跳过", self.get_node_name());
            return;
        }

        // 将is_processing_order设置为true
        self.set_is_processing_order(true).await;
        tracing::info!("{}: 开始创建订单", self.get_node_id());
        let mut virtual_trading_system_guard = self.virtual_trading_system.lock().await;
        let virtual_order_id = virtual_trading_system_guard.create_order(
            self.get_strategy_id().clone(),
            self.get_node_id().clone(),
            self.backtest_config.order_config.symbol.clone(),
            self.backtest_config.order_config.price,
            self.backtest_config.order_config.order_side.clone(),
            self.backtest_config.order_config.order_type.clone(),
            self.backtest_config.order_config.quantity,
            self.backtest_config.order_config.tp,
            self.backtest_config.order_config.sl,
        );

        let all_order = virtual_trading_system_guard.get_orders();
        tracing::info!("{}: 所有订单: {:?}", self.get_node_id(), all_order);
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
        unfilled_virtual_order: Arc<RwLock<Vec<VirtualOrder>>>,
        is_processing_order: Arc<RwLock<bool>>,
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

    async fn get_strategy_cache_keys(&mut self) -> Result<Vec<CacheKey>, String> {
        let (tx, rx) = oneshot::channel();
        let node_command = NodeCommand::GetStrategyCacheKeys(GetStrategyCacheKeysParams {
            node_id: self.get_node_id().clone(),
            timestamp: Utc::now().timestamp_millis(),
            responder: tx,
        });

        self.get_strategy_command_sender().send(node_command).await.unwrap();

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
                let kline_cache_keys = cache_keys.iter().filter(|k| matches!(k, CacheKey::BacktestKline(_))).collect::<Vec<&CacheKey>>();
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
            let cache_key = BacktestKlineCacheKey::new(
                self.backtest_config.exchange_config.as_ref().unwrap().selected_data_source.exchange.clone(),
                self.backtest_config.order_config.symbol.clone(),
                min_kline_interval.clone(),
                self.backtest_config.exchange_config.as_ref().unwrap().time_range.start_date.to_string(),
                self.backtest_config.exchange_config.as_ref().unwrap().time_range.end_date.to_string(),
            );

            // 这里需要减去1 ，因为信号发送后，strategy中会+1 ， 而这里需要获取+1前的index
            let kline_cache_index = *self.kline_cache_index.read().await;

            tracing::info!("{}: 获取K线缓存数据, index: {:?}", self.get_node_id(), kline_cache_index);
            let (tx, rx) = oneshot::channel();
            let get_cache_params = GetCacheParams {
                strategy_id: self.base_context.strategy_id.clone(),
                node_id: self.base_context.node_id.clone(),
                cache_key: cache_key.clone().into(),
                index: Some(kline_cache_index),
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

}

#[async_trait]
impl BacktestNodeContextTrait for OrderNodeContext {
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
        self.base_context.output_handle.get(&format!("order_node_output")).unwrap().clone()
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

    async fn handle_node_event(&mut self, node_event: NodeEvent) -> Result<(), String> {
        match node_event {
            NodeEvent::Signal(signal_event) => {
                match signal_event {
                    SignalEvent::ConditionMatch(_) => {
                        tracing::debug!("{}: 收到信号事件: {:?}", self.get_node_id(), signal_event);
                        self.get_kline_cache_data().await;
                        // self.create_order().await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) -> Result<(), String> {
        match strategy_inner_event {
            StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
                // 更新k线缓存索引
                *self.kline_cache_index.write().await = play_index_update_event.played_index;
                tracing::debug!("{}: 更新k线缓存索引: {}", self.get_node_id(), play_index_update_event.played_index);
            }
            _ => {}
        }
        Ok(())
    }

}


