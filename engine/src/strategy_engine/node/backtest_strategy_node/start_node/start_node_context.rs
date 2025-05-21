use crate::strategy_engine::node::node_context::{BaseNodeContext, NodeContextTrait};
use std::any::Any;
use event_center::{command::cache_engine_command::{GetCacheLengthMultiParams, GetCacheLengthParams, CacheEngineCommand}, Event};
use tracing::instrument;
use types::strategy::{node_command::{GetStrategyCacheKeysParams, StrategyCommand}, node_message::{NodeMessage, SignalMessage, SignalType}};
use async_trait::async_trait;
use types::strategy::BacktestStrategyConfig;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use std::sync::Arc;
use tokio::sync::RwLock;
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use types::cache::CacheKey;
use utils::get_utc8_timestamp_millis;
use std::collections::HashMap;
use event_center::response::cache_engine_response::CacheEngineResponse;
use types::strategy::node_response::{NodeResponse, StrategyResponse};

#[derive(Debug, Clone)]
pub struct StartNodeContext {
    pub base_context: BaseNodeContext,
    pub backtest_config: Arc<RwLock<BacktestStrategyConfig>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub strategy_cache_keys: Vec<CacheKey>,
    pub cache_lengths: HashMap<CacheKey, u32>,
    
}


#[async_trait]
impl NodeContextTrait for StartNodeContext {

    fn clone_box(&self) -> Box<dyn NodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn get_base_context(&self) -> &BaseNodeContext {
        &self.base_context
    }
    
    fn get_base_context_mut(&mut self) -> &mut BaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("start_node_output")).unwrap().clone()
    }


    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        tracing::info!("{}: 收到事件: {:?}", self.base_context.node_id, event);
        Ok(())
    }
    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
        Ok(())
    }
    
}

impl StartNodeContext {
    // 发送信号(每发送一次信号，k线节点就拉取一次数据)
    pub async fn send_signal(&self) {
        let node_id = self.base_context.node_id.clone();
        let node_name = self.base_context.node_name.clone();
        let output_handle = self.get_default_output_handle();
        let backtest_config = self.backtest_config.clone();

        // 获取所有缓存的最小长度
        let min_cache_length = self.cache_lengths.values().min().cloned().unwrap_or(0);



        let heartbeat = self.heartbeat.lock().await;
        heartbeat.run_async_task_once(
            format!("{}发送信号", self.base_context.node_name),
            async move {
                Self::send_fetch_kline_data_signal(node_id, node_name, output_handle, backtest_config, min_cache_length).await;
            }
        ).await;
    }

    // 获取策略缓存key(向strategy发送命令)
    #[instrument(skip(self))]
    pub async fn get_strategy_cache_key(&mut self) -> Result<Vec<CacheKey>, String> {
        tracing::info!(strategy_id = self.base_context.strategy_id, "get strategy cache key");
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_strategy_cache_keys_params = GetStrategyCacheKeysParams {
            node_id: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx
        };
        let strategy_command = StrategyCommand::GetStrategyCacheKeys(get_strategy_cache_keys_params);
        self.get_strategy_command_sender().send(strategy_command.into()).await.unwrap();
        let response = resp_rx.await.unwrap();
        if response.code() == 0 {
            let strategy_response = StrategyResponse::try_from(response);
            if let Ok(strategy_response) = strategy_response {
                match strategy_response {
                    StrategyResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                        tracing::info!(cache_keys = ?get_strategy_cache_keys_response.cache_keys, "get strategy cache keys successfully!");
                        Ok(get_strategy_cache_keys_response.cache_keys)
                    }
                    // _ => Err("get strategy cache keys failed".to_string())
                }
            } else {
                Err("try from response failed".to_string())
            }
        } else {
            Err("get strategy cache keys failed".to_string())
        }
    }

    #[instrument(skip(self))]
    // 获取所有k线缓存中的最小长度
    pub async fn get_cache_length(&self) -> Result<HashMap<CacheKey, u32>, String> {
        
        // 过滤出k线缓存key
        let kline_cache_keys = self.strategy_cache_keys
            .iter()
            .filter(|cache_key| matches!(cache_key, CacheKey::HistoryKline(_)))
            .map(|cache_key| cache_key.clone())
            .collect();
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_cache_length_params = GetCacheLengthMultiParams {
            strategy_id: self.base_context.strategy_id.clone(),
            cache_keys: kline_cache_keys,
            timestamp: get_utc8_timestamp_millis(),
            sender: self.base_context.node_id.clone(),
            responder: resp_tx
        };
        let cache_engine_command = CacheEngineCommand::GetCacheLengthMulti(get_cache_length_params);
        // 向缓存引擎发送命令
        self.get_command_publisher().send(cache_engine_command.into()).await.unwrap();
        let response = resp_rx.await.unwrap();
        if response.code() == 0 {
            let cache_engine_response = CacheEngineResponse::try_from(response);
            if let Ok(cache_engine_response) = cache_engine_response {
                match cache_engine_response {
                    CacheEngineResponse::GetCacheLengthMulti(get_cache_length_multi_response) => {
                        tracing::info!(cache_lengths = ?get_cache_length_multi_response.cache_length, "Get cache length successfully!");
                        Ok(get_cache_length_multi_response.cache_length)
                    }
                    _ => Err("get cache length multi failed".to_string())
                }
            } else {
                Err("try from response failed".to_string())
            }

        } else {
            Err("get cache length multi failed".to_string())
        }

    }


    // 发送信号(每发送一次信号，k线节点就拉取一次数据)
    async fn send_fetch_kline_data_signal(
        node_id: String,
        node_name: String,
        output_handle: NodeOutputHandle,
        backtest_config: Arc<RwLock<BacktestStrategyConfig>>,
        signal_count: u32, // 信号计数
    ) {
        
        // 根据信号计数发送信号
        for i in 0..signal_count {
            // 获取当前播放速度
            let play_speed = {
                let config = backtest_config.read().await;
                let speed = config.play_speed;
                
                // 确保 play_speed 在合理范围内（1-100）
                if speed < 1 {
                    tracing::warn!("播放速度小于1，已调整为1");
                    1
                } else if speed > 100 {
                    tracing::warn!("播放速度大于100，已调整为100");
                    100
                } else {
                    speed
                }
            };
            let fetch_kline_message = SignalMessage {
                from_node_id: node_id.clone(),
                from_node_name: node_name.clone(),
                from_node_handle_id: output_handle.output_handle_id.clone(),
                signal_type: SignalType::FetchKlineData(i),
                message_timestamp: chrono::Utc::now().timestamp_millis(),
            };
            
            let signal = NodeMessage::Signal(fetch_kline_message.clone());
            // tracing::info!("{}: 发送信号: {:?}", node_id, signal);
            output_handle.send(signal).unwrap();
            
            // 根据播放速度计算延迟时间（毫秒）
            let delay_millis = 1000 / play_speed as u64;
            
            // 休眠指定时间
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_millis)).await;
            
            // 继续循环，每次循环都会重新读取最新的播放速度
        }
    }
}
