pub mod backtest_strategy_context;
pub mod backtest_strategy_state_machine;
pub mod backtest_strategy_function;
pub mod backtest_strategy_control;


use std::sync::Arc;
use tokio::sync::RwLock;
use backtest_strategy_context::BacktestStrategyContext;
use backtest_strategy_state_machine::{BacktestStrategyStateAction, BacktestStrategyStateMachine};
use types::strategy::Strategy;
use event_center::EventPublisher;
use tokio::sync::{Mutex, Notify};
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use petgraph::Graph;
use std::collections::HashMap;
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use backtest_strategy_function::BacktestStrategyFunction;
use crate::strategy_engine::{node::BacktestNodeTrait, strategy::backtest_strategy::backtest_strategy_state_machine::*};
use types::cache::CacheKey;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use types::strategy::BacktestStrategyConfig;
use tokio::sync::{mpsc, broadcast};
use types::strategy::node_command::NodeCommand;
use virtual_trading::VirtualTradingSystem;
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use super::super::node::backtest_strategy_node::start_node::StartNode;
use super::StrategyCommandPublisher;
use types::virtual_trading_system::event::VirtualTradingSystemEvent;

#[derive(Debug, Clone)]
pub struct BacktestStrategy {
    pub context: Arc<RwLock<BacktestStrategyContext>>,
}


impl BacktestStrategy {
    pub async fn new(
        strategy: Strategy,
        event_publisher: EventPublisher, 
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        market_event_receiver: EventReceiver, 
        response_event_receiver: EventReceiver,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>
    ) -> Self {
        let mut graph = Graph::new();
        let mut node_indices = HashMap::new();
        let mut cache_keys: Vec<CacheKey> = vec![];
        let mut strategy_command_publisher = StrategyCommandPublisher::new();

        // 创建虚拟交易系统
        let (virtual_trading_system_event_tx, virtual_trading_system_event_rx) = broadcast::channel::<VirtualTradingSystemEvent>(100);
        let virtual_trading_system = Arc::new(Mutex::new(VirtualTradingSystem::new(command_publisher.clone(), virtual_trading_system_event_tx)));

        let strategy_id = strategy.id;
        let strategy_name = strategy.name;
        let (node_command_tx, node_command_rx) = mpsc::channel::<NodeCommand>(100);
        // 创建策略内部事件的广播通道
        let (strategy_inner_event_tx, strategy_inner_event_rx) = broadcast::channel::<StrategyInnerEvent>(100);


        // 当策略创建时，状态为 Created
        let cancel_token = CancellationToken::new();
        let cancel_play_token = CancellationToken::new();

        
        if let Some(nodes_str) = strategy.nodes {
            if let Ok(nodes) = serde_json::from_str::<Vec<Value>>(&nodes_str.to_string()) {
                for node_config in nodes {
                    tracing::debug!("添加节点: {:?}", node_config);
                    BacktestStrategyFunction::add_node(
                        &mut graph, 
                        &mut node_indices, 
                        &mut cache_keys,
                        node_config, 
                        event_publisher.clone(), 
                        command_publisher.clone(),
                        command_receiver.clone(),
                        market_event_receiver.resubscribe(), 
                        response_event_receiver.resubscribe(),
                        database.clone(),
                        heartbeat.clone(),
                        &mut strategy_command_publisher,
                        node_command_tx.clone(),
                        virtual_trading_system.clone(),
                        strategy_inner_event_rx.resubscribe(),
                        virtual_trading_system_event_rx.resubscribe()
                    ).await.unwrap();

                }
            }
        }
        // 添加边
        if let Some(edges_str) = strategy.edges {
            if let Ok(edges) = serde_json::from_str::<Vec<Value>>(&edges_str.to_string()) {
                // tracing::debug!("edges: {:?}", edges);
                for edge_config in edges {
                    let from_handle_id = edge_config["sourceHandle"].as_str().unwrap();
                    let from_node_id = edge_config["source"].as_str().unwrap();
                    let to_node_id = edge_config["target"].as_str().unwrap();
                    let to_handle_id = edge_config["targetHandle"].as_str().unwrap();

                    BacktestStrategyFunction::add_edge(&mut graph, &mut node_indices, from_node_id, from_handle_id, to_node_id, to_handle_id).await;
                    
                }
            }
        }

        // 将所有节点的输出控制器添加到 strategy_output_handles 中
        let strategy_output_handles = BacktestStrategyFunction::add_strategy_output_handle(&mut graph).await;
        tracing::debug!("all node's strategy output handles: {:?}", strategy_output_handles);
        
        
        // tracing::debug!("策略的输出句柄: {:?}", strategy_output_handles);
        tracing::debug!("virtual trading system kline cache keys: {:?}", virtual_trading_system.lock().await.kline_price);
        let context = BacktestStrategyContext {
            strategy_id,
            strategy_name: strategy_name.clone(),
            cache_keys: Arc::new(RwLock::new(cache_keys)),
            cache_lengths: HashMap::new(),
            graph,
            node_indices,
            event_publisher,
            event_receivers: vec![response_event_receiver],
            cancel_token,
            state_machine: BacktestStrategyStateMachine::new(strategy_id, strategy_name, BacktestStrategyRunState::Created),
            all_node_output_handles: strategy_output_handles, // 所有节点的输出控制器
            database: database,
            heartbeat: heartbeat,
            registered_tasks: Arc::new(RwLock::new(HashMap::new())),
            command_publisher: command_publisher,
            command_receiver: command_receiver,
            node_command_receiver: Arc::new(Mutex::new(node_command_rx)),
            strategy_command_publisher,
            signal_count: Arc::new(RwLock::new(0)),
            played_signal_index: Arc::new(RwLock::new(0)),
            is_playing: Arc::new(RwLock::new(false)),
            initial_play_speed: Arc::new(RwLock::new(0)),
            cancel_play_token: cancel_play_token,
            virtual_trading_system: virtual_trading_system,
            strategy_inner_event_publisher: strategy_inner_event_tx,
            updated_play_index_node_ids: Arc::new(RwLock::new(vec![])),
            updated_play_index_notify: Arc::new(Notify::new()),
        };
        Self { context: Arc::new(RwLock::new(context)) }
    }
}


impl BacktestStrategy {

    pub fn get_context(&self) -> Arc<RwLock<BacktestStrategyContext>> {
        self.context.clone()
    }

    pub async fn get_strategy_id(&self) -> i32 {
        self.context.read().await.strategy_id
    }

    pub async fn get_strategy_name(&self) -> String {
        self.context.read().await.strategy_name.clone()
    }

    pub async fn get_state_machine(&self) -> BacktestStrategyStateMachine {
        self.context.read().await.state_machine.clone()
    }

    pub async fn update_strategy_state(&mut self, event: BacktestStrategyStateTransitionEvent) -> Result<(), String> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let strategy_name = self.get_strategy_name().await;

        let (transition_result, state_machine) = {
            let mut state_manager = self.context.read().await.state_machine.clone();
            let transition_result = state_manager.transition(event).unwrap();
            (transition_result, state_manager)
        };

        tracing::info!("需要执行的动作: {:?}", transition_result.actions);
        for action in transition_result.actions {
            match action {
                BacktestStrategyStateAction::InitInitialPlaySpeed => {
                    tracing::info!("{}: 初始化初始播放速度", strategy_name);
                    let context_guard = self.context.read().await;
                    let start_node_config = context_guard.get_start_node_config().await;
                    if let Ok(start_node_config) = start_node_config {
                        let mut initial_play_speed_guard = context_guard.initial_play_speed.write().await;
                        *initial_play_speed_guard = start_node_config.play_speed as u32;
                        tracing::info!("{}: 初始化初始播放速度成功。播放速度: {:?}", strategy_name, *initial_play_speed_guard);
                    } else {
                        tracing::error!("{}: 获取start节点配置失败", strategy_name);
                    }
                    

                    // let backtest_config = context_guard.strategy_config.clone();
                    // let initial_play_speed = backtest_config.play_speed as u32;
                    // tracing::info!("{}: 初始化初始播放速度成功。播放速度: {:?}", strategy_name, backtest_config);

                    // let mut initial_play_speed_guard = context_guard.initial_play_speed.write().await;
                    // tracing::info!("{}: 初始化初始播放速度成功。播放速度: {}", strategy_name, initial_play_speed);
                    // *initial_play_speed_guard = initial_play_speed;
                }
                BacktestStrategyStateAction::InitSignalCount => {
                    tracing::info!("{}: 初始化信号计数", strategy_name);
                    let mut context_guard = self.context.write().await;
                    let signal_count = context_guard.get_signal_count().await;
                    if let Ok(signal_count) = signal_count {
                        let mut signal_count_guard = context_guard.signal_count.write().await;
                        *signal_count_guard = signal_count;
                        tracing::info!("{}: 初始化信号计数成功", strategy_name);
                    } else {
                        tracing::error!("{}: 获取信号计数失败", strategy_name);
                    }
                }
                BacktestStrategyStateAction::InitCacheLength => {
                    tracing::info!("{}: 初始化缓存长度", strategy_name);
                    let mut context_guard = self.context.write().await;
                    let cache_lengths = context_guard.get_cache_length().await;
                    if let Ok(cache_lengths) = cache_lengths {
                        context_guard.cache_lengths = cache_lengths;
                    } else {
                        tracing::error!("{}: 获取缓存长度失败", strategy_name);
                    }
                }

                BacktestStrategyStateAction::InitNode => {
                    
                    let strategy_id = self.get_strategy_id().await;
                    tracing::info!(strategy_id = %strategy_id, strategy_name = %strategy_name, "start init node");
                    let nodes = {
                        let context_guard = self.context.read().await;
                        context_guard.topological_sort()
                    };
                    
                    let mut all_nodes_initialized = true;

                    for node in nodes {
                        let context_guard = self.context.read().await;
                        if let Err(e) = context_guard.init_node(node).await {
                            tracing::error!("{}", e);
                            all_nodes_initialized = false;
                            break;
                        }
                    }

                    if all_nodes_initialized {
                        tracing::info!("{}: 所有节点已成功初始化", strategy_name);
                    } else {
                        tracing::error!("{}: 部分节点初始化失败，策略无法正常运行", strategy_name);
                    }
                }

                BacktestStrategyStateAction::StopNode => {
                    tracing::info!("++++++++++++++++++++++++++++++++++++++");
                    tracing::info!("{}: 开始停止节点", strategy_name);
                    let nodes = {
                        let context_guard = self.context.read().await;
                        context_guard.topological_sort()
                    };
                    
                    let mut all_nodes_stopped = true;

                    for node in nodes {
                        // let mut node = node.clone();
                        let context_guard = self.context.read().await;
                        
                        if let Err(e) = context_guard.stop_node(node).await {
                            tracing::error!("{}", e);
                            all_nodes_stopped = false;
                            break;
                        }
                    }

                    if all_nodes_stopped {
                        tracing::info!("{}: 所有节点已成功停止", strategy_name);
                    } else {
                        tracing::error!("{}: 部分节点停止失败，策略无法正常运行", strategy_name);
                    }
                }
                
                BacktestStrategyStateAction::LogTransition => {
                    tracing::info!("{}: 状态转换: {:?} -> {:?}", strategy_name, self.get_state_machine().await.current_state(), transition_result.new_state);
                }

                BacktestStrategyStateAction::ListenAndHandleNodeEvent => {
                    tracing::info!("{}: 监听节点消息", strategy_name);
                    BacktestStrategyFunction::listen_node_events(self.get_context()).await;
                }
                BacktestStrategyStateAction::ListenAndHandleNodeCommand => {
                    tracing::info!("{}: 监听命令", strategy_name);
                    BacktestStrategyFunction::listen_node_command(self.get_context()).await;
                }
                BacktestStrategyStateAction::LogError(error) => {
                    tracing::error!("{}: {}", strategy_name, error);
                }
            }

            {
                let mut context_guard = self.context.write().await;
                context_guard.set_state_machine(state_machine.clone());
            }
            

            
        }
        Ok(())
        

    }

    
    pub async fn init_strategy(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始初始化策略", self.get_strategy_name().await);

        // created => initializing
        self.update_strategy_state(BacktestStrategyStateTransitionEvent::Initialize).await.unwrap();

        // 
        // initializing => ready
        tracing::info!("{}: 初始化完成", self.get_strategy_name().await);
        self.update_strategy_state(BacktestStrategyStateTransitionEvent::InitializeComplete).await.unwrap();

        Ok(())
    }

    pub async fn stop_strategy(&mut self) -> Result<(), String> {
        // 获取当前状态
        // 如果策略当前状态为 Stopped，则不进行操作
        let current_state = self.get_state_machine().await.current_state();
        if current_state == BacktestStrategyRunState::Stopping {
            tracing::info!("策略{}已停止", self.get_strategy_name().await);
            return Ok(());
        }
        tracing::info!("等待所有节点停止...");
        self.update_strategy_state(BacktestStrategyStateTransitionEvent::Stop).await.unwrap();

        // 发送完信号后，循环遍历所有的节点，获取节点的状态，如果所有的节点状态都为stopped，则更新策略状态为Stopped
        let all_stopped = {
            let context_guard = self.context.read().await;
            context_guard.wait_for_all_nodes_stopped(10).await.unwrap()
        };
        if all_stopped {
            self.update_strategy_state(BacktestStrategyStateTransitionEvent::StopComplete).await.unwrap();
            Ok(())
        } else {
            Err("等待节点停止超时".to_string())
        }
    }

    pub async fn play(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始播放k线", self.get_strategy_name().await);
        let context_guard = self.context.read().await;
        context_guard.play().await;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始暂停策略", self.get_strategy_name().await);
        let mut context_guard = self.context.write().await;
        context_guard.pause().await;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), String> {
        tracing::info!("{}: 重置播放", self.get_strategy_name().await);
        let mut context_guard = self.context.write().await;
        context_guard.reset().await;
        Ok(())
    }

    pub async fn play_one_kline(&mut self) -> Result<u32, String> {
        
        let context_guard = self.context.read().await;
        let played_signal_count = context_guard.play_one_kline().await;
        if let Ok(played_signal_count) = played_signal_count {
            Ok(played_signal_count)
        } else {
            Err("播放单根k线失败".to_string())
        }
    }
    
}