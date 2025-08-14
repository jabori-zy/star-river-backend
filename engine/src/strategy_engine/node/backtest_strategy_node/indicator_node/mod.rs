pub mod indicator_node_state_machine;
pub mod indicator_node_context;
pub mod indicator_node_type;

use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::strategy_engine::node::{BacktestNodeTrait,NodeType};
use crate::strategy_engine::node::node_state_machine::*;
use indicator_node_state_machine::{IndicatorNodeStateManager,IndicatorNodeStateAction};
use std::time::Duration;
use indicator_node_context::IndicatorNodeContext;
use event_center::EventPublisher;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use tokio::sync::Mutex;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver, command::backtest_strategy_command::StrategyCommandReceiver};
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use indicator_node_type::IndicatorNodeBacktestConfig;
use types::cache::key::{IndicatorKey, KlineKey};
use types::strategy::node_event::BacktestNodeEvent;
use types::custom_type::PlayIndex;

// 指标节点
#[derive(Debug, Clone)]
pub struct IndicatorNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
    
}




impl IndicatorNode {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String, 
        backtest_config: IndicatorNodeBacktestConfig,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        node_command_sender: NodeCommandSender,
        strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Self {

        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::IndicatorNode,
            event_publisher,
            vec![response_event_receiver],
            command_publisher,
            command_receiver,
            Box::new(IndicatorNodeStateManager::new(BacktestNodeRunState::Created, node_id, node_name)),
            node_command_sender,
            strategy_command_receiver,
            strategy_inner_event_receiver,
            play_index_watch_rx
        );

        // 通过配置，获取指标缓存键
        let indicator_cache_keys = Self::get_indicator_cache_keys(&backtest_config);
        tracing::debug!("indicator_cache_keys: {:?}", indicator_cache_keys);
        // 通过配置，获取回测K线缓存键
        let kline_cache_key = Self::get_kline_cache_key(&backtest_config);

        Self {
            context: Arc::new(RwLock::new(Box::new(IndicatorNodeContext {
                base_context,
                backtest_config,
                is_registered: Arc::new(RwLock::new(false)),
                indicator_cache_keys,
                kline_cache_key,
            }))),
            
        }
    }

    fn get_indicator_cache_keys(backtest_config: &IndicatorNodeBacktestConfig) -> Vec<IndicatorKey> {
        let exchange = backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.exchange.clone();
        let symbol = backtest_config.exchange_mode_config.as_ref().unwrap().selected_symbol.symbol.clone();
        let interval = backtest_config.exchange_mode_config.as_ref().unwrap().selected_symbol.interval.clone();
        let time_range = backtest_config.exchange_mode_config.as_ref().unwrap().time_range.clone();

        let mut indicator_keys = vec![];
        for indicator in backtest_config.exchange_mode_config.as_ref().unwrap().selected_indicators.iter() {
            let indicator_cache_key = IndicatorKey {
                exchange: exchange.clone(), 
                symbol: symbol.clone(), 
                interval: interval.clone(), 
                indicator_config: indicator.indicator_config.clone(),
                start_time: Some(time_range.start_date.to_string()),
                end_time: Some(time_range.end_date.to_string()),
            };
            indicator_keys.push(indicator_cache_key);
        }
        indicator_keys
    }

    fn get_kline_cache_key(backtest_config: &IndicatorNodeBacktestConfig) -> KlineKey {
        let exchange = backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.exchange.clone();
        let symbol = backtest_config.exchange_mode_config.as_ref().unwrap().selected_symbol.symbol.clone();
        let interval = backtest_config.exchange_mode_config.as_ref().unwrap().selected_symbol.interval.clone();
        let time_range = backtest_config.exchange_mode_config.as_ref().unwrap().time_range.clone();

        let kline_cache_key = KlineKey {
            exchange: exchange.clone(), 
            symbol: symbol.clone(), 
            interval: interval.clone(), 
            start_time: Some(time_range.start_date.to_string()),
            end_time: Some(time_range.end_date.to_string()),
        };
        kline_cache_key
    }
    
    
}

#[async_trait]
impl BacktestNodeTrait for IndicatorNode {

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn BacktestNodeTrait> {
        Box::new(self.clone())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>> {
        self.context.clone()
    }

    // 设置节点的出口
    async fn set_output_handle(&mut self) {
        
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;

        // 添加strategy_ouput_handle
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!(node_id = %node_id, node_name = %node_name, strategy_output_handle_id = %strategy_output_handle_id, "setting strategy output handle");
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        self.add_output_handle(strategy_output_handle_id, tx).await;
        
        

        // 添加默认出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let default_output_handle_id = format!("{}_default_output", node_id);
        tracing::debug!(node_id = %node_id, node_name = %node_name, default_output_handle_id = %default_output_handle_id, "setting default output handle");
        self.add_output_handle(default_output_handle_id, tx).await;

        // 添加每一个indicator的出口
        let selected_indicator = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let indicator_node_context = context_guard.as_any().downcast_ref::<IndicatorNodeContext>().unwrap();
            let exchange_mode_config = indicator_node_context.backtest_config.exchange_mode_config.as_ref().unwrap();
            exchange_mode_config.selected_indicators.clone()
        };
        
        for indicator in selected_indicator.iter() {
            let indicator_output_handle_id = indicator.handle_id.clone();
            tracing::debug!(node_id = %node_id, node_name = %node_name, indicator_output_handle_id = %indicator_output_handle_id, "setting indicator output handle");
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(indicator_output_handle_id, tx).await;
        }
        tracing::info!(node_id = %node_id, node_name = %node_name, "setting node handle complete");
    }

    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.context.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize).await.unwrap();

        // 循环检查是否已经注册指标
        // 检查交易所是否注册成功，并且K线流是否订阅成功
        loop {
            let is_registered = {
                let state_guard = self.context.read().await;
                let indicator_node_context = state_guard.as_any().downcast_ref::<IndicatorNodeContext>().unwrap();
                let is_registered = indicator_node_context.is_registered.read().await.clone();
                tracing::info!("{}: 检查是否已经注册指标: {}", self.get_node_id().await, is_registered);
                is_registered
            };
            if is_registered {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        tracing::info!("{:?}: 初始化完成", self.context.read().await.get_state_machine().current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete).await?;

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始停止", self.get_node_id().await);
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop).await.unwrap();

        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }

    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), String> {
        let node_id = self.get_node_id().await;
        
        // 获取状态管理器并执行转换
        let (transition_result, state_machine) = {
            let mut state_machine = self.get_state_machine().await;
            let transition_result = state_machine.transition(event)?;
            (transition_result, state_machine)
        };
        
        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            if let Some(indicator_node_state_action) = action.as_any().downcast_ref::<IndicatorNodeStateAction>() {
                match indicator_node_state_action {
                    IndicatorNodeStateAction::LogTransition => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    IndicatorNodeStateAction::LogNodeState => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    IndicatorNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    IndicatorNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("{}: 开始监听节点传递的message", node_id);
                        self.listen_node_events().await?;
                    }
                    IndicatorNodeStateAction::ListenAndHandleInnerEvents => {
                        tracing::info!("{}: 开始监听策略内部事件", node_id);
                        self.listen_strategy_inner_events().await?;
                    }
                    IndicatorNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!("{}: 开始监听策略命令", node_id);
                        self.listen_strategy_command().await?;
                    }
                    IndicatorNodeStateAction::ListenAndHandlePlayIndex => {
                        tracing::info!("{}: 开始监听播放索引", node_id);
                        self.listen_play_index().await?;
                    }
                    IndicatorNodeStateAction::RegisterIndicatorCacheKey => {
                        tracing::info!("{}: 开始注册指标缓存键", node_id);
                        let mut context = self.context.write().await;
                        let context = context.as_any_mut().downcast_mut::<IndicatorNodeContext>().unwrap();
                        let is_all_success = context.register_indicator_cache_key().await;
                        if let Ok(is_all_success) = is_all_success {
                            if is_all_success {
                                *context.is_registered.write().await = true;
                                tracing::info!("{}: 注册指标缓存键成功", node_id);
                            } else {
                                tracing::error!("{}: 注册指标缓存键失败", node_id);
                            }
                        }
                    }
                    IndicatorNodeStateAction::CalculateIndicator => {
                        tracing::info!("{}: 开始计算指标", node_id);
                        let mut context = self.context.write().await;
                        let context = context.as_any_mut().downcast_mut::<IndicatorNodeContext>().unwrap();
                        let is_all_success = context.calculate_indicator().await;
                        if let Ok(is_all_success) = is_all_success {
                            if is_all_success {
                                tracing::info!("{}: 计算指标成功", node_id);
                            } else {
                                tracing::error!("{}: 计算指标失败", node_id);
                            }
                        } else {
                            tracing::error!("{}: 计算指标失败", node_id);
                        }
                    }
                    IndicatorNodeStateAction::CancelAsyncTask => {
                        tracing::debug!(node_id = %node_id, "cancel async task");
                        self.cancel_task().await;
                    }
                    _ => {}
                }
                // 所有动作执行完毕后更新节点最新的状态
                {
                    self.context.write().await.set_state_machine(state_machine.clone_box());
                }
            }
        }
                    
        Ok(())
    }
}