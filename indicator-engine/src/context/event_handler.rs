use super::IndicatorEngineContext;
use event_center::event::Event;
use engine_core::context_trait::EngineEventHandler;
use async_trait::async_trait;
use event_center::communication::engine::EngineCommand;
use event_center::communication::engine::indicator_engine::IndicatorEngineCommand;
use event_center::communication::engine::indicator_engine::*;
use std::sync::Arc;
use event_center::communication::Command;
use ta_lib::TALib;
use crate::calculate::CalculateIndicatorFunction;

#[async_trait]
impl EngineEventHandler for IndicatorEngineContext {

    async fn handle_event(&mut self, _event: Event) {
        // if let Event::Indicator(indicator_event) = event {
        //     match indicator_event {
        //         _ => {}
        //     }
        // }
    }

    async fn handle_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::IndicatorEngine(indicator_engine_command) => {
                match indicator_engine_command {
                    IndicatorEngineCommand::GetIndicatorLookback(cmd) => {
                        let lookback = TALib::lookback(&cmd.indicator_key.indicator_config);
                        let payload = GetIndicatorLookbackRespPayload::new(cmd.indicator_key.clone(), lookback);

                        let response = GetIndicatorLookbackResponse::success(Some(payload));
                        cmd.respond(response);
                    }
                    // 计算指标
                    IndicatorEngineCommand::CalculateHistoryIndicator(cmd) => {
                        let cal_result =
                            CalculateIndicatorFunction::calculate_indicator(cmd.kline_series.clone(), cmd.indicator_config.clone()).await;
                        match cal_result {
                            Ok(indicators) => {
                                let payload = CalculateHistoryIndicatorRespPayload::new(
                                    cmd.kline_key.clone(),
                                    cmd.indicator_config.clone(),
                                    indicators,
                                );
                                let response = CalculateHistoryIndicatorResponse::success(Some(payload));
                                cmd.respond(response);
                            }
                            Err(error) => {
                                let error = Arc::new(error);
                                let response = CalculateHistoryIndicatorResponse::error(error);
                                cmd.respond(response);
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}