use super::IndicatorEngineContext;
use crate::indicator_engine::calculate::CalculateIndicatorFunction;
use crate::{EngineContext, EngineName};
use async_trait::async_trait;
use event_center::communication::engine::EngineCommand;
use event_center::communication::engine::EngineResponse;
use event_center::communication::engine::indicator_engine::*;
use event_center::event::Event;
use event_center::event::ExchangeEvent;
use star_river_core::cache::key::IndicatorKey;
use std::any::Any;
use std::sync::Arc;

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

    async fn handle_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::IndicatorEngine(indicator_engine_command) => {
                match indicator_engine_command {
                    // 注册指标, 并且初始化
                    IndicatorEngineCommand::RegisterIndicator(register_indicator_params) => {
                        self.register_indicator(
                            register_indicator_params.strategy_id,
                            register_indicator_params.node_id.clone(),
                            register_indicator_params.exchange.clone(),
                            register_indicator_params.symbol.clone(),
                            register_indicator_params.interval.clone(),
                            register_indicator_params.indicator_config.clone(),
                        )
                        .await;
                        // 发送注册指标完成事件
                        let register_indicator_response = RegisterIndicatorResponse::success(
                            register_indicator_params.strategy_id,
                            register_indicator_params.node_id,
                            register_indicator_params.exchange,
                            register_indicator_params.symbol,
                            register_indicator_params.interval,
                            register_indicator_params.indicator_config,
                        );
                        let response_event = IndicatorEngineResponse::RegisterIndicator(register_indicator_response);
                        register_indicator_params.responder.send(response_event.into()).unwrap();
                    }
                    // 计算指标
                    IndicatorEngineCommand::CalculateHistoryIndicator(cal_history_ind_params) => {
                        let backtest_indicators = CalculateIndicatorFunction::calculate_indicator(
                            self.cache_engine.clone(),
                            cal_history_ind_params.kline_key.clone().into(),
                            cal_history_ind_params.indicator_config.clone(),
                            true, //一次性将历史数据计算出来
                        )
                        .await
                        .unwrap();
                        // 将指标数据添加到缓存中
                        let backtest_indicator_key = self
                            .cache_engine
                            .lock()
                            .await
                            .initialize_indicator_cache(
                                cal_history_ind_params.kline_key.clone().into(),
                                cal_history_ind_params.indicator_config.clone(),
                                backtest_indicators,
                            )
                            .await;
                        // 发送计算指标完成响应
                        let calculate_backtest_indicator_response =
                            CalculateHistoryIndicatorResponse::success(backtest_indicator_key);
                        let response_event =
                            IndicatorEngineResponse::CalculateHistoryIndicator(calculate_backtest_indicator_response);
                        cal_history_ind_params.responder.send(response_event.into()).unwrap();
                    }
                    // 计算指标
                    IndicatorEngineCommand::CalculateIndicator(cal_ind_params) => {
                        // 计算结果
                        let calculate_result = CalculateIndicatorFunction::calculate_indicator(
                            self.cache_engine.clone(),
                            cal_ind_params.kline_key.clone().into(),
                            cal_ind_params.indicator_config.clone(),
                            false,
                        )
                        .await;
                        match calculate_result {
                            Ok(indicator) => {
                                // 更新缓存
                                let cache_engine_guard = self.cache_engine.lock().await;
                                cache_engine_guard
                                    .update_indicator_cache(
                                        cal_ind_params.kline_key.clone(),
                                        cal_ind_params.indicator_config.clone(),
                                        indicator.last().unwrap().clone(),
                                    )
                                    .await;
                                // 发送计算指标完成响应
                                let indicator_key =
                                    IndicatorKey::new(cal_ind_params.kline_key, cal_ind_params.indicator_config);
                                let calculate_indicator_response: EngineResponse = CalculateIndicatorResponse::success(
                                    indicator_key.into(),
                                    indicator.last().unwrap().clone(),
                                )
                                .into();
                                cal_ind_params.responder.send(calculate_indicator_response).unwrap();
                            }
                            Err(error) => {
                                let indicator_key =
                                    IndicatorKey::new(cal_ind_params.kline_key, cal_ind_params.indicator_config);
                                let calculate_indicator_response: EngineResponse =
                                    CalculateIndicatorResponse::error(Arc::new(error), indicator_key.into()).into();
                                cal_ind_params.responder.send(calculate_indicator_response).unwrap();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
