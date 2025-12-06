use std::sync::Arc;

use async_trait::async_trait;
use engine_core::context_trait::EngineEventHandler;
use event_center::{EngineCommand, Event};
use star_river_event::communication::indicator_engine::{
    CalculateIndicatorResponse, CalculateLookbackRespPayload, CalculateLookbackResponse, CalculateRespPayload, IndicatorEngineCommand,
};
use ta_lib::TALib;

use super::IndicatorEngineContext;
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
                    IndicatorEngineCommand::CalculateLookback(cmd) => {
                        let lookback = TALib::lookback(&cmd.indicator_key.indicator_config);
                        let payload = CalculateLookbackRespPayload::new(cmd.indicator_key.clone(), lookback);

                        let response = CalculateLookbackResponse::success(payload);
                        cmd.respond(response);
                    }
                    // Calculate indicator
                    IndicatorEngineCommand::CalculateIndicator(cmd) => {
                        let cal_result =
                            CalculateIndicatorFunction::calculate_indicator(cmd.kline_series.clone(), cmd.indicator_config.clone()).await;
                        match cal_result {
                            Ok(indicators) => {
                                let payload = CalculateRespPayload::new(cmd.kline_key.clone(), cmd.indicator_config.clone(), indicators);
                                let response = CalculateIndicatorResponse::success(payload);
                                cmd.respond(response);
                            }
                            Err(error) => {
                                let response = CalculateIndicatorResponse::fail(Arc::new(error));
                                cmd.respond(response);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
