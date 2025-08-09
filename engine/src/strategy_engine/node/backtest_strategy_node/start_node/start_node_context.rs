use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use std::any::Any;
use event_center::{command::cache_engine_command::{GetCacheLengthMultiParams, GetCacheLengthParams, CacheEngineCommand}, Event};
use tracing::instrument;
use types::strategy::{node_command::{GetStrategyCacheKeysParams}, node_event::{BacktestNodeEvent}};
use async_trait::async_trait;
use types::strategy::BacktestStrategyConfig;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use std::sync::Arc;
use tokio::sync::RwLock;
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use utils::get_utc8_timestamp_millis;
use types::strategy::node_event::{KlinePlayEvent, KlinePlayFinishedEvent, SignalEvent, PlayIndexUpdateEvent};
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use event_center::command::backtest_strategy_command::StrategyCommand;
use event_center::response::backtest_strategy_response::{StrategyResponse, GetStartNodeConfigResponse};

#[derive(Debug, Clone)]
pub struct StartNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: Arc<RwLock<BacktestStrategyConfig>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    
}


#[async_trait]
impl BacktestNodeContextTrait for StartNodeContext {

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
        self.base_context.output_handles.get(&format!("start_node_default_output")).unwrap().clone()
    }


    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        tracing::info!("{}: 收到事件: {:?}", self.base_context.node_id, event);
        Ok(())
    }
    async fn handle_node_event(&mut self, message: BacktestNodeEvent) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
        Ok(())
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) -> Result<(), String> {
        match strategy_inner_event {
            StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
                // 更新播放索引
                self.set_play_index(play_index_update_event.play_index).await;
                let strategy_output_handle = self.get_strategy_output_handle();
                // 更新完成后，发送索引已更新事件
                let signal = BacktestNodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
                    from_node_id: self.get_node_id().clone(),
                    from_node_name: self.get_node_name().clone(),
                    from_node_handle_id: strategy_output_handle.output_handle_id.clone(),
                    play_index: self.get_play_index().await,
                    message_timestamp: get_utc8_timestamp_millis(),
                }));
                strategy_output_handle.send(signal).unwrap();
            },
            StrategyInnerEvent::NodeReset => {
                tracing::info!("{}: 收到节点重置事件", self.base_context.node_id);
            }
        }
        Ok(())
    }

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) -> Result<(), String> {
        tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
        match strategy_command {
            StrategyCommand::GetStartNodeConfig(get_start_node_config_params) => {
                let start_node_config = self.backtest_config.read().await.clone();
                
                let response = StrategyResponse::GetStartNodeConfig(GetStartNodeConfigResponse {
                    code: 0,
                    message: "success".to_string(),
                    node_id: self.base_context.node_id.clone(),
                    backtest_strategy_config: start_node_config,
                    response_timestamp: get_utc8_timestamp_millis(),
                });

                get_start_node_config_params.responder.send(response).unwrap();
                
            }
        }
        Ok(())
    }
    
}

impl StartNodeContext {
    // 发送k线跳动信号
    pub async fn send_play_signal(&self, play_index : i32) {
        let kline_tick_event = KlinePlayEvent {
            from_node_id: self.base_context.node_id.clone(),
            from_node_name: self.base_context.node_name.clone(),
            from_node_handle_id: "start_node_default_output".to_string(),
            play_index: play_index,
            message_timestamp: chrono::Utc::now().timestamp_millis(),
        };
        
        let signal = BacktestNodeEvent::Signal(SignalEvent::KlinePlay(kline_tick_event.clone()));
        // 通过default出口，给节点发送信号
        self.get_default_output_handle().send(signal.clone()).unwrap();

    }

    // 发送k线播放完毕信号
    pub async fn send_finish_signal(&self, play_index : i32) {
        let default_output_handle = self.get_default_output_handle();
        let finish_signal = KlinePlayFinishedEvent {
            from_node_id: self.get_node_id().clone(),
            from_node_name: self.get_node_name().clone(),
            from_node_handle_id: default_output_handle.output_handle_id.clone(),
            play_index,
            message_timestamp: get_utc8_timestamp_millis(),
        };

        let signal = BacktestNodeEvent::Signal(SignalEvent::KlinePlayFinished(finish_signal));
        default_output_handle.send(signal.clone()).unwrap();

    }
}
