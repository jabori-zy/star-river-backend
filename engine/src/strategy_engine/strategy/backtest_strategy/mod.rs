pub mod backtest_strategy_context;
pub mod backtest_strategy_state_machine;
pub mod backtest_strategy_function;


use std::sync::Arc;
use tokio::sync::RwLock;
use backtest_strategy_context::BacktestStrategyContext;
use crate::strategy_engine::strategy::StrategyTrait;
use crate::strategy_engine::strategy::strategy_context::StrategyContext;
use async_trait::async_trait;
use std::any::Any;
use crate::strategy_engine::strategy::strategy_state_machine::{StrategyStateTransitionEvent,StrategyStateMachine};
use backtest_strategy_state_machine::{BacktestStrategyStateAction, BacktestStrategyStateMachine};
use types::strategy::Strategy;
use event_center::EventPublisher;
use tokio::sync::Mutex;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use petgraph::Graph;
use std::collections::HashMap;
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use backtest_strategy_function::BacktestStrategyFunction;
// use backtest_strategy_state_machine::BacktestStrategyStateMachine;
use crate::strategy_engine::strategy::strategy_state_machine::StrategyRunState;
use types::cache::CacheKey;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use types::strategy::BacktestStrategyConfig;
use tokio::sync::mpsc;
use types::strategy::node_command::NodeCommand;
use crate::strategy_engine::strategy::strategy_functions::StrategyFunction;
use super::super::node::node_types::NodeType;
use super::super::node::backtest_strategy_node::start_node::StartNode;


#[derive(Debug, Clone)]
pub struct BacktestStrategy {
    pub context: Arc<RwLock<Box<dyn StrategyContext>>>,
}


impl BacktestStrategy {
    pub async fn new(
        strategy: Strategy,
        event_publisher: EventPublisher, 
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        market_event_receiver: EventReceiver, 
        response_event_receiver: EventReceiver,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>
    ) -> Self {
        let mut graph = Graph::new();
        let mut node_indices = HashMap::new();
        let mut strategy_backtest_config = BacktestStrategyConfig::default();
        let mut cache_keys: Vec<CacheKey> = vec![];

        let strategy_id = strategy.id;
        let strategy_name = strategy.name;
        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<NodeCommand>(100);


        // 当策略创建时，状态为 Created
        let cancel_token = CancellationToken::new();


        match strategy.config {
            Some(config) => {
                let backtest_config = config["backtestConfig"].clone();
                if let Ok(backtest_config) = serde_json::from_value::<BacktestStrategyConfig>(backtest_config) {
                    strategy_backtest_config = backtest_config;
                } else {
                    tracing::error!("策略配置解析失败");
                }
            }
            None => {
                tracing::warn!("策略配置为空");
            }
        }

        
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
                        exchange_engine.clone(),
                        database.clone(),
                        heartbeat.clone(),
                        strategy_command_tx.clone(),
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
        let strategy_output_handles = BacktestStrategyFunction::add_node_output_handle(&mut graph).await;
        
        
        tracing::debug!("策略的输出句柄: {:?}", strategy_output_handles);
        let context = BacktestStrategyContext {
            strategy_id,
            strategy_name: strategy_name.clone(),
            strategy_config: strategy_backtest_config,
            cache_keys: Arc::new(RwLock::new(cache_keys)),
            graph,
            node_indices,
            event_publisher,
            event_receivers: vec![response_event_receiver],
            cancel_token,
            state_machine: Box::new(BacktestStrategyStateMachine::new(strategy_id, strategy_name, StrategyRunState::Created)),
            all_node_output_handles: strategy_output_handles, // 所有节点的输出控制器
            positions: Arc::new(RwLock::new(vec![])),
            exchange_engine: exchange_engine,
            database: database,
            heartbeat: heartbeat,
            registered_tasks: Arc::new(RwLock::new(HashMap::new())),
            command_publisher: command_publisher,
            command_receiver: command_receiver,
            strategy_command_receiver: Arc::new(Mutex::new(strategy_command_rx)),
        };
        Self { context: Arc::new(RwLock::new(Box::new(context))) }
    }
}




#[async_trait]
impl StrategyTrait for BacktestStrategy {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn StrategyTrait> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn StrategyContext>>> {
        self.context.clone()
    }

    async fn get_strategy_id(&self) -> i32 {
        self.context.read().await.get_strategy_id()
    }

    async fn get_strategy_name(&self) -> String {
        self.context.read().await.get_strategy_name()
    }

    async fn get_state_machine(&self) -> Box<dyn StrategyStateMachine> {
        self.context.read().await.get_state_machine()
    }

    async fn update_strategy_state(&mut self, event: StrategyStateTransitionEvent) -> Result<(), String> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let strategy_name = self.get_strategy_name().await;

        let (transition_result, state_machine) = {
            let mut state_manager = self.context.read().await.get_state_machine().clone_box();
            let transition_result = state_manager.transition(event).unwrap();
            (transition_result, state_manager)
        };

        tracing::info!("需要执行的动作: {:?}", transition_result.get_actions());
        for action in transition_result.get_actions() {
            if let Some(backtest_strategy_state_action) = action.as_any().downcast_ref::<BacktestStrategyStateAction>() {
                match backtest_strategy_state_action {
                    BacktestStrategyStateAction::InitNode => {
                        tracing::info!("++++++++++++++++++++++++++++++++++++++");
                        tracing::info!("{}: 开始初始化节点", strategy_name);
                        let nodes = {
                            let context_guard = self.context.read().await;
                            let backtest_strategy_context = context_guard.as_any().downcast_ref::<BacktestStrategyContext>().unwrap();
                            backtest_strategy_context.topological_sort()
                        };
                        
                        let mut all_nodes_initialized = true;

                        for node in nodes {
                            let context_guard = self.context.read().await;
                            let backtest_strategy_context = context_guard.as_any().downcast_ref::<BacktestStrategyContext>().unwrap();
                            if let Err(e) = backtest_strategy_context.init_node(node).await {
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

                    BacktestStrategyStateAction::StartNode => {
                        tracing::info!("++++++++++++++++++++++++++++++++++++++");
                        tracing::info!("{}: 开始启动节点", strategy_name);
                        let nodes = {
                            let context_guard = self.context.read().await;
                            let backtest_strategy_context = context_guard.as_any().downcast_ref::<BacktestStrategyContext>().unwrap();
                            backtest_strategy_context.topological_sort()
                        };
                        
                        let mut all_nodes_started = true;

                        for node in nodes.iter() {
                            if let Err(e) = StrategyFunction::start_node(node).await {
                                tracing::error!("{}", e);
                                all_nodes_started = false;
                                break;
                            }
                        }

                        if all_nodes_started {
                            tracing::info!("{}: 所有节点已成功启动", strategy_name);
                            // 找到开始节点
                            for node in nodes {
                                if node.get_node_type().await == NodeType::StartNode {
                                    let start_node = node.as_any().downcast_ref::<StartNode>().unwrap();
                                    start_node.send_signal().await;
                                    
                                    break;
                                }
                            }
                        } else {
                            tracing::error!("{}: 部分节点启动失败，策略无法正常运行", strategy_name);
                        }
                    }


                    BacktestStrategyStateAction::StopNode => {
                        tracing::info!("++++++++++++++++++++++++++++++++++++++");
                        tracing::info!("{}: 开始停止节点", strategy_name);
                        let nodes = {
                            let context_guard = self.context.read().await;
                            let backtest_strategy_context = context_guard.as_any().downcast_ref::<BacktestStrategyContext>().unwrap();
                            backtest_strategy_context.topological_sort()
                        };
                        
                        let mut all_nodes_stopped = true;

                        for node in nodes {
                            // let mut node = node.clone();
                            let context_guard = self.context.read().await;
                            let backtest_strategy_context = context_guard.as_any().downcast_ref::<BacktestStrategyContext>().unwrap();
                            
                            if let Err(e) = backtest_strategy_context.stop_node(node).await {
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
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", strategy_name, self.get_state_machine().await.current_state(), transition_result.get_new_state());
                    }
                    BacktestStrategyStateAction::RegisterTask => {
                        tracing::info!("{}: 注册任务", strategy_name);
                        let mut context_guard = self.context.write().await;
                        let backtest_strategy_context = context_guard.as_any_mut().downcast_mut::<BacktestStrategyContext>().unwrap();
                        backtest_strategy_context.monitor_positions().await;
                    }
                    BacktestStrategyStateAction::LoadPositions => {
                        tracing::info!("{}: 加载持仓", strategy_name);
                        let mut context_guard = self.context.write().await;
                        let backtest_strategy_context = context_guard.as_any_mut().downcast_mut::<BacktestStrategyContext>().unwrap();
                        backtest_strategy_context.load_all_positions().await;
                    }
                    BacktestStrategyStateAction::ListenAndHandleNodeMessage => {
                        tracing::info!("{}: 监听节点消息", strategy_name);
                        self.listen_node_message().await.unwrap();
                    }
                    BacktestStrategyStateAction::ListenAndHandleCommand => {
                        tracing::info!("{}: 监听命令", strategy_name);
                        self.listen_command().await.unwrap();
                    }

                    BacktestStrategyStateAction::ListenAndHandleEvent => {
                        tracing::info!("{}: 监听事件", strategy_name);
                        self.listen_event().await.unwrap();
                    }
                    _ => {}
                }
            }

            {
                let mut context_guard = self.context.write().await;
                context_guard.set_state_machine(state_machine.clone_box());
            }
            

            
        }
        Ok(())

    }

    
    async fn init_strategy(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始初始化策略", self.get_strategy_name().await);

        // created => initializing
        self.update_strategy_state(StrategyStateTransitionEvent::Initialize).await.unwrap();

        // 
        // initializing => ready
        tracing::info!("{}: 初始化完成", self.get_strategy_name().await);
        self.update_strategy_state(StrategyStateTransitionEvent::InitializeComplete).await.unwrap();

        Ok(())
    }

    async fn start_strategy(&mut self) -> Result<(), String> {
        tracing::info!("启动策略: {}", self.get_strategy_name().await);
        // 获取当前状态
        let current_state = self.get_state_machine().await.current_state();
        // 如果当前状态为 Running，则不进行操作
        if current_state != StrategyRunState::Ready {
            tracing::info!("策略未处于Ready状态, 不能启动: {}", self.get_strategy_name().await);
            return Ok(());
        }
        // 策略发送启动信号

        tracing::info!("{}: 发送启动策略信号", self.get_strategy_name().await);
        tracing::info!("等待所有节点启动...");
        self.update_strategy_state(StrategyStateTransitionEvent::Start).await.unwrap();

        // 先获取是否所有节点都在运行的结果，然后释放不可变借用
        let all_running = {
            let context_guard = self.context.read().await;
            let backtest_strategy_context = context_guard.as_any().downcast_ref::<BacktestStrategyContext>().unwrap();
            backtest_strategy_context.wait_for_all_nodes_running(10).await.unwrap()
        };
        
        if all_running {
            self.update_strategy_state(StrategyStateTransitionEvent::StartComplete).await.unwrap();
            Ok(())
        } else {
            Err("等待节点启动超时".to_string())
        }
    }

    async fn stop_strategy(&mut self) -> Result<(), String> {
        // 获取当前状态
        // 如果策略当前状态为 Stopped，则不进行操作
        let current_state = self.get_state_machine().await.current_state();
        if current_state == StrategyRunState::Stopping {
            tracing::info!("策略{}已停止", self.get_strategy_name().await);
            return Ok(());
        }
        tracing::info!("等待所有节点停止...");
        self.update_strategy_state(StrategyStateTransitionEvent::Stop).await.unwrap();

        // 发送完信号后，循环遍历所有的节点，获取节点的状态，如果所有的节点状态都为stopped，则更新策略状态为Stopped
        let all_stopped = {
            let context_guard = self.context.read().await;
            let backtest_strategy_context = context_guard.as_any().downcast_ref::<BacktestStrategyContext>().unwrap();
            backtest_strategy_context.wait_for_all_nodes_stopped(10).await.unwrap()
        };
        if all_stopped {
            self.update_strategy_state(StrategyStateTransitionEvent::StopComplete).await.unwrap();
            Ok(())
        } else {
            Err("等待节点停止超时".to_string())
        }
    }

    async fn enable_strategy_data_push(&mut self) -> Result<(), String> {
        let mut context_guard = self.context.write().await;
        let backtest_strategy_context = context_guard.as_any_mut().downcast_mut::<BacktestStrategyContext>().unwrap();
        backtest_strategy_context.enable_strategy_data_push().await;
        Ok(())
    }

    async fn disable_strategy_data_push(&mut self) -> Result<(), String> {
        let mut context_guard = self.context.write().await;
        let backtest_strategy_context = context_guard.as_any_mut().downcast_mut::<BacktestStrategyContext>().unwrap();
        backtest_strategy_context.disable_strategy_data_push().await;
        Ok(())
    }
    


}