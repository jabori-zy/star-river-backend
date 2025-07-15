use event_center::response::indicator_engine_response::{RegisterIndicatorResponse, IndicatorEngineResponse, CalculateBacktestIndicatorResponse};
use tokio::sync::broadcast;
use event_center::Event;
use async_trait::async_trait;
use std::any::Any;
use std::time::Duration;
use crate::{EngineContext, EngineName};
use event_center::command::Command;
use event_center::command::indicator_engine_command::IndicatorEngineCommand;
use utils::get_utc8_timestamp_millis;
use event_center::EventPublisher;
use types::indicator::IndicatorConfig;
use tokio::sync::Mutex;
use std::sync::Arc;
use types::cache::key::{IndicatorKey, BacktestKlineKey, KlineKey};
use types::cache::Key;
use types::custom_type::{StrategyId, NodeId};
use crate::cache_engine::CacheEngine;
use crate::indicator_engine::indicator_engine_type::IndicatorSubKey;
use std::collections::HashMap;
use event_center::exchange_event::ExchangeEvent;
use event_center::exchange_event::ExchangeKlineUpdateEvent;
use types::market::{Exchange, KlineInterval};
use heartbeat::Heartbeat;
use crate::indicator_engine::calculate::CalculateIndicatorFunction;
use event_center::{EventReceiver, CommandPublisher, CommandReceiver};

#[derive(Debug)]
pub struct IndicatorEngineContext {
    pub engine_name: EngineName,
    pub cache_engine: Arc<Mutex<CacheEngine>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub event_publisher: EventPublisher,
    pub command_publisher: CommandPublisher,
    pub command_receiver: Arc<Mutex<CommandReceiver>>,
    pub event_receiver: Vec<EventReceiver>,
    pub subscribe_indicators: Arc<Mutex<HashMap<IndicatorSubKey, Vec<StrategyId>>>>, // 已订阅的指标
    
}


impl Clone for IndicatorEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            cache_engine: self.cache_engine.clone(),
            heartbeat: self.heartbeat.clone(),
            subscribe_indicators: self.subscribe_indicators.clone(),
            command_publisher: self.command_publisher.clone(),
            command_receiver: self.command_receiver.clone(),
        }
    }
}


#[async_trait]
impl EngineContext for IndicatorEngineContext {

    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(self.clone())
    }


    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_engine_name(&self) -> EngineName {
        self.engine_name.clone()
    }

    fn get_event_publisher(&self) -> &EventPublisher {
        &self.event_publisher
    }

    fn get_event_receiver(&self) -> Vec<broadcast::Receiver<Event>> {
        self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect()
    }

    fn get_command_publisher(&self) -> &CommandPublisher {
        &self.command_publisher
    }

    fn get_command_receiver(&self) -> Arc<Mutex<CommandReceiver>> {
        self.command_receiver.clone()
    }

    async fn handle_event(&mut self, event: Event) {
        if let Event::Exchange(exchange_event) = event {
            match exchange_event {
                // 接收到k线更新事件， 触发指标计算
                ExchangeEvent::ExchangeKlineUpdate(exchange_kline_update_event) => {
                    // 处理k线更新事件， 触发指标计算
                    self.handle_exchange_kline_update(exchange_kline_update_event).await;
                }
                _ => {}
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::IndicatorEngine(indicator_engine_command) => {
                match indicator_engine_command {
                    // 注册指标, 并且初始化
                    IndicatorEngineCommand::RegisterIndicator(register_indicator_params) => {
                        self.register_indicator(
                            register_indicator_params.strategy_id, 
                            register_indicator_params.node_id.clone(), 
                            register_indicator_params.exchange.clone(), 
                            register_indicator_params.symbol.clone(), 
                            register_indicator_params.interval.clone(), 
                            register_indicator_params.indicator_config.clone()
                        ).await;
                        // 发送注册指标完成事件
                        let register_indicator_response = RegisterIndicatorResponse {
                            code: 0,
                            message: "success".to_string(),
                            strategy_id: register_indicator_params.strategy_id,
                            node_id: register_indicator_params.node_id,
                            exchange: register_indicator_params.exchange,
                            symbol: register_indicator_params.symbol,
                            interval: register_indicator_params.interval,
                            indicator: register_indicator_params.indicator_config,
                            response_timestamp: get_utc8_timestamp_millis(),
                        };
                        let response_event = IndicatorEngineResponse::RegisterIndicator(register_indicator_response);
                        register_indicator_params.responder.send(response_event.into()).unwrap();
                    }
                    // 计算指标
                    IndicatorEngineCommand::CalculateBacktestIndicator(calculate_backtest_indicator_params) => {
                        let backtest_indicators = CalculateIndicatorFunction::calculate_indicator(
                            self.cache_engine.clone(), 
                            calculate_backtest_indicator_params.kline_cache_key.clone().into(),
                            calculate_backtest_indicator_params.indicator_config.clone(),
                            true //一次性将历史数据计算出来
                        ).await.unwrap();
                        // 将指标数据添加到缓存中
                        let backtest_indicator_cache_key = self.cache_engine.lock().await.initialize_backtest_indicator_cache(
                            calculate_backtest_indicator_params.kline_cache_key.clone().into(),
                            calculate_backtest_indicator_params.indicator_config.clone(),
                            backtest_indicators
                        ).await;
                        // 发送计算指标完成响应
                        let calculate_backtest_indicator_response = CalculateBacktestIndicatorResponse {
                            code: 0,
                            message: "success".to_string(),
                            backtest_indicator_cache_key: backtest_indicator_cache_key,
                            response_timestamp: get_utc8_timestamp_millis(),
                        };
                        let response_event = IndicatorEngineResponse::CalculateBacktestIndicator(calculate_backtest_indicator_response);
                        calculate_backtest_indicator_params.responder.send(response_event.into()).unwrap();
                    }
                }
            }
            _ => {}
        }
    }

}


impl IndicatorEngineContext {
    // k线更新后， 触发指标计算
    async fn handle_exchange_kline_update(&mut self, exchange_kline_update_event: ExchangeKlineUpdateEvent) {
        // tracing::info!("接收到k线更新事件: {:?},当前订阅指标: {:?}", exchange_kline_update_event, self.subscribe_indicators.lock().await);
        // 遍历订阅的指标， 计算指标
        // 判断是否需要计算指标
        let kline_exchange = exchange_kline_update_event.exchange.clone();
        let kline_symbol = exchange_kline_update_event.symbol.clone();
        let kline_interval = exchange_kline_update_event.interval.clone();
        // 判断是否需要计算指标
        let should_calculate = self.should_calculate(kline_exchange.clone(), kline_symbol.clone(), kline_interval.clone()).await;
        // 如果需要计算指标，则获取需要计算的指标
        if should_calculate {
            // 获取需要计算的指标
            let indicator_sub_keys = {
                let sub_indicators = self.subscribe_indicators.lock().await.clone();
                let indicator_sub_keys = sub_indicators
                        .keys()
                        .cloned()
                        .filter(|sub_key| sub_key.exchange == kline_exchange && sub_key.symbol == kline_symbol && sub_key.interval == kline_interval) // 过滤出需要计算的指标
                        .collect::<Vec<IndicatorSubKey>>();
                indicator_sub_keys
            };
            // 计算指标
            for indicator_sub_key in indicator_sub_keys {
                let cache_engine = self.cache_engine.clone();
                // 注册任务
                let indicator_sub_key_clone = indicator_sub_key.clone();
                let futures = async move {
                    let kline_cache_key = KlineKey::new(
                        indicator_sub_key_clone.exchange.clone(), 
                        indicator_sub_key_clone.symbol.clone(), 
                        indicator_sub_key_clone.interval.clone()
                    );

                    let indicators = CalculateIndicatorFunction::calculate_indicator(
                        cache_engine.clone(), 
                        kline_cache_key.into(),
                        indicator_sub_key_clone.indicator_config.clone(), 
                        false
                    ).await.unwrap();
                    let last_indicator = indicators.last().unwrap();
                    // 将指标添加到缓存中
                    let cache_engine_guard = cache_engine.lock().await;
                    cache_engine_guard.update_indicator_cache(
                        indicator_sub_key_clone.exchange, 
                        indicator_sub_key_clone.symbol, 
                        indicator_sub_key_clone.interval, 
                        indicator_sub_key_clone.indicator_config,
                        last_indicator.clone()).await;
                };
                let heartbeat = self.heartbeat.lock().await;
                heartbeat.run_async_task_once(
                    format!("calculate_indicator_{}", indicator_sub_key.indicator_config.to_string()), 
                    futures
                ).await;

            }
        }
    }

    // 判断是否需要计算指标
    async fn should_calculate(&self, kline_exchange: Exchange, kline_symbol: String, kline_interval: KlineInterval) -> bool {
        let sub_indicators = self.subscribe_indicators.lock().await.clone();
            // 判断指标subkey的exchange, symbol, interval是否与k线更新事件的exchange, symbol, interval相同
        let mut should_calculate = false;
        for (sub_key, _) in sub_indicators.iter() {
            if sub_key.exchange == kline_exchange && sub_key.symbol == kline_symbol && sub_key.interval == kline_interval {
                should_calculate = true;
                break;
            }
        }
        should_calculate
    }

    // 注册指标
    async fn register_indicator(&mut self,
        strategy_id: StrategyId,
        node_id: NodeId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        indicator_config: IndicatorConfig,
    ) {
        // tracing::info!("接收到注册指标命令: {:?}", register_indicator_params);

        // 1. 将指标添加到已订阅的指标列表中,策略也添加到已订阅的策略列表中
        let indicator_sub_key = IndicatorSubKey {
            exchange: exchange.clone(),
            symbol: symbol.clone(),
            interval: interval.clone(),
            indicator_config: indicator_config.clone(),
        };
        let mut subscribe_indicators = self.subscribe_indicators.lock().await;
        subscribe_indicators.entry(indicator_sub_key.clone()).or_insert(vec![]).push(strategy_id);
        tracing::info!("已订阅的指标: {:?}", subscribe_indicators);
        
        // 1. 添加缓存键
        let indicator_cache_key: IndicatorKey = indicator_sub_key.clone().into();
        let _ = self.cache_engine.lock().await.add_cache_key(indicator_cache_key.into(), None, Duration::from_millis(10)).await;
        // 3. 计算指标
        let kline_cache_key = KlineKey::new(
            exchange.clone(), 
            symbol.clone(), 
            interval.clone()
        );
        let indicators = CalculateIndicatorFunction::calculate_indicator(
            self.cache_engine.clone(), 
            kline_cache_key.into(),
            indicator_sub_key.indicator_config.clone(), 
            true
        ).await.unwrap();
        // tracing::info!("计算得到的指标: {:?}", indicators);
        // 4. 将指标添加到缓存中
        self.cache_engine.lock().await.initialize_indicator_cache(exchange.clone(), symbol.clone(), interval.clone(), indicator_config.clone(), indicators).await;
    }


}