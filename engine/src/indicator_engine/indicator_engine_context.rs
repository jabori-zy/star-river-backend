use tokio::sync::broadcast;
use event_center::Event;
use async_trait::async_trait;
use std::any::Any;
use crate::{EngineName,EngineContext};
use event_center::command_event::CommandEvent;
use event_center::command_event::indicator_engine_command::{CalculateIndicatorParams, IndicatorEngineCommand};
use event_center::response_event::indicator_engine_response::{IndicatorEngineResponse, CalculateIndicatorResponse};
use utils::get_utc8_timestamp_millis;
use event_center::response_event::ResponseEvent;
use event_center::EventPublisher;
use types::indicator::Indicators;
use crate::indicator_engine::talib::TALib;
use types::indicator::IndicatorValue;
use types::indicator::SMAIndicator;
use types::indicator_config::SMAConfig;
use std::collections::HashMap;




#[derive(Debug)]
pub struct IndicatorEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,

}


impl Clone for IndicatorEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),

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

    async fn handle_event(&mut self, event: Event) {
        match event {
            // Event::Response(response_event) => {
            //     self.handle_response_event(response_event).await;
            // }
            Event::Command(command_event) => {
                match command_event {
                    CommandEvent::IndicatorEngine(indicator_engine_command) => {
                        match indicator_engine_command {
                            IndicatorEngineCommand::CalculateIndicator(calculate_indicator_params) => {
                                self.calculate_indicator(calculate_indicator_params).await;
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


impl IndicatorEngineContext {

    async fn calculate_indicator(&self, calculate_indicator_params: CalculateIndicatorParams) {
        // tracing::info!("接收到计算指标命令: {:?}", calculate_indicator_params);
        let indicator = calculate_indicator_params.indicator.clone();

        match indicator {
            Indicators::SimpleMovingAverage(sma_config) => {
                let period = sma_config.period;
                self.calculate_sma(&period, calculate_indicator_params).await.unwrap();
            }
        }


        
    }

    async fn calculate_sma(&self, period: &i32, calculate_params: CalculateIndicatorParams) -> Result<(), String> {
        let kline_series = calculate_params.kline_series.clone();
        let timestamp_list: Vec<i64> = kline_series.series.iter().map(|v| v.timestamp).collect();  
        let close: Vec<f64> = kline_series.series.iter().map(|v| v.close).collect();

        let sma = TALib::sma(&close, *period)?;
        // log::info!("{}: sma: {:?}", event.symbol,sma);
        // 将timestamp_list和sma组合成SMA结构体
        let sma_list: Vec<IndicatorValue> = timestamp_list.iter().zip(sma.iter()).map(|(timestamp, sma)| IndicatorValue { timestamp: *timestamp, value: *sma }).collect();
        // log::info!("{}: sma_list: {:?}", kline_event.symbol,sma_list);
        
        let sma_series = SMAIndicator {
            exchange: calculate_params.exchange.clone(),
            symbol: calculate_params.symbol.clone(),
            kline_interval: calculate_params.interval.clone(),
            indicator_config: SMAConfig { period: *period },
            indicator_value: HashMap::from([("sma".to_string(), sma_list)]),
        };

        let response = CalculateIndicatorResponse {
            exchange: calculate_params.exchange.clone(),
            symbol: calculate_params.symbol.clone(),
            interval: calculate_params.interval.clone(),
            indicator: calculate_params.indicator.clone(),
            value: Box::new(sma_series),
            response_timestamp: get_utc8_timestamp_millis(),
            response_id: calculate_params.request_id.clone(),
            batch_id: calculate_params.batch_id.clone(),
        };
        
        let response_event = ResponseEvent::IndicatorEngine(IndicatorEngineResponse::CalculateIndicatorFinish(response));
        self.get_event_publisher().publish(response_event.clone().into()).unwrap();
        // tracing::debug!("响应事件: {:?}", response_event);

        Ok(())
    }

    fn payload_to_list(payload: &serde_json::Value) -> Vec<f64> {
        let list = payload["list"].as_array().unwrap();
        list.iter().map(|v| v.as_f64().unwrap()).collect()
    }

}