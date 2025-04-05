mod state_manager;

use event_center::command_event::{SubscribeKlineStreamParams, MarketDataEngineCommand, CommandEvent, UnsubscribeKlineStreamParams};
use types::market::{Exchange, KlineInterval};
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use event_center::market_event::MarketEvent;
use crate::*;
use types::strategy::message::{KlineSeriesMessage, NodeMessage};
use tokio::sync::RwLock;
use std::sync::Arc;
use uuid::Uuid;
use event_center::EventPublisher;
use event_center::command_event::{RegisterExchangeParams, ExchangeManagerCommand};
use event_center::response_event::{MarketDataEngineResponse, ResponseEvent, ExchangeManagerResponse};
use event_center::strategy_event::StrategyEvent;
use std::collections::HashMap;
use crate::NodeOutputHandle;
use crate::node::NodeRunState;
use crate::node::NodeTrait;
use crate::node::NodeStateTransitionEvent;
use crate::node::state_machine::NodeStateMachine;
use crate::node::live_data_node::state_manager::{LiveDataNodeStateMachine, LiveDataNodeStateAction};
use crate::node::base_node_state::BaseNodeState;
use crate::node::node_functions::NodeFunction;
use crate::node::base_node_state::NodeState;
use tokio_util::sync::CancellationToken;

// 将需要共享的状态提取出来
#[derive(Debug)]
pub struct LiveDataNodeState {
    pub base_state: BaseNodeState<LiveDataNodeStateMachine>,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
    pub is_subscribed: bool,
    pub request_id: Option<Uuid>,
}

#[async_trait]
impl NodeState for LiveDataNodeState {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_event_receivers(&self) -> &Vec<broadcast::Receiver<Event>> {
        &self.base_state.event_receivers
    }
    fn get_cancel_token(&self) -> &CancellationToken {
        &self.base_state.cancel_token
    }
    fn get_node_id(&self) -> &String {
        &self.base_state.node_id
    }
    fn get_node_name(&self) -> &String {
        &self.base_state.node_name
    }
    fn get_strategy_id(&self) -> &i32 {
        &self.base_state.strategy_id
    }
    fn get_run_state(&self) -> NodeRunState {
        self.base_state.run_state_manager.current_state()
    }
    fn get_output_handle(&self) -> &HashMap<String, NodeOutputHandle> {
        &self.base_state.output_handle
    }
    fn get_output_handle_mut(&mut self) -> &mut HashMap<String, NodeOutputHandle> {
        &mut self.base_state.output_handle
    }
    fn enable_event_publish(&self) -> &bool {
        &self.base_state.enable_event_publish
    }
    fn get_state_machine(&self) -> Box<dyn NodeStateMachine> {
        Box::new(self.base_state.run_state_manager.clone())
    }
    
    
    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        match event {
            Event::Market(market_event) => {
                self.handle_market_event(market_event).await;
            }
            Event::Response(response_event) => {
                self.handle_response_event(response_event).await;
            }
            _ => {}
        }
        Ok(())
    }
    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_state.node_id, message);
        Ok(())
    }
    
}


impl LiveDataNodeState {

    async fn handle_market_event(&self, market_event: MarketEvent) {
        // 先获取读锁，检查状态
        // let state_guard = self.base_state.clone();

        if self.base_state.run_state_manager.current_state() != NodeRunState::Running {
            tracing::warn!("{}: 节点状态不是Running, 不处理行情数据", self.base_state.node_id);
            return;
        }

        // 处理市场事件
        match market_event {
            MarketEvent::KlineSeriesUpdate(kline_series_update) => {
                // 只获取当前节点支持的数据
                let exchange = self.exchange.clone();
                let symbol = self.symbol.clone();
                let interval = self.interval.clone();
                if exchange != kline_series_update.exchange || symbol != kline_series_update.symbol || interval != kline_series_update.interval {
                    return;
                }
                // 这里不需要再获取锁，因为我们只需要读取数据
                let kline_series_message = KlineSeriesMessage {
                    from_node_id: self.base_state.node_id.clone(),
                    from_node_name: self.base_state.node_name.clone(),
                    exchange: kline_series_update.exchange,
                    symbol: kline_series_update.symbol,
                    interval: kline_series_update.interval,
                    kline_series: kline_series_update.kline_series.clone(),
                    batch_id: kline_series_update.batch_id.clone(),
                    message_timestamp: get_utc8_timestamp_millis(),
                };
                
                let message = NodeMessage::KlineSeries(kline_series_message);
                tracing::info!("{}: 发送数据: {:?}", self.base_state.node_id, message);
                // 获取handle的连接数
                let default_handle_connect_count = self.base_state.output_handle.get("live_data_node_output").expect("实时数据节点默认的消息发送器不存在").sender.receiver_count();
                // 如果连接数为0，则不发送数据
                if default_handle_connect_count > 0 {
                    let default_node_sender = self.base_state.output_handle.get("live_data_node_output").expect("实时数据节点默认的消息发送器不存在");
                    // tracing::info!("{}: 发送数据: {:?}", state_guard.node_id, message);
                    match default_node_sender.sender.send(message.clone()) {
                        Ok(_) => (),
                        Err(e) => tracing::error!(
                            node_id = %self.base_state.node_id,
                            error = ?e,
                            receiver_count = default_node_sender.sender.receiver_count(),
                                "数据源节点发送数据失败"
                            ),
                        }
                    
                }

                // 发送事件
                if self.base_state.enable_event_publish {
                    let event = Event::Strategy(StrategyEvent::NodeMessage(message));
                    if let Err(_) = self.base_state.event_publisher.publish(event.into()) {
                        tracing::error!(
                            node_id = %self.base_state.node_id,
                            "数据源节点发送数据事件失败"
                        );
                    }
                }

            }
            _ => {}
        }
    }

    async fn handle_response_event(&mut self, response_event: ResponseEvent) {
        tracing::info!("{}: 收到响应事件: {:?}", self.base_state.node_id, response_event);
        
        let request_id= {
            match self.request_id {
                Some(id) => {
                    tracing::info!("{}: 当前的请求id: {:?}", self.base_state.node_id, id);
                    id
                },
                None => {
                    tracing::warn!("{}: 请求id不存在", self.base_state.node_id);
                    return;
                }
            }
        };


        match response_event {
            ResponseEvent::ExchangeManager(ExchangeManagerResponse::RegisterExchangeSuccess(register_exchange_success_response)) => {
                if request_id == register_exchange_success_response.response_id {
                    tracing::info!("{}: 交易所注册成功: {:?}", self.base_state.node_id, register_exchange_success_response);
                    self.request_id = None;
                }
            }
            ResponseEvent::MarketDataEngine(MarketDataEngineResponse::SubscribeKlineStreamSuccess(subscribe_kline_stream_success_response)) => {
                
                if request_id == subscribe_kline_stream_success_response.response_id {
                    tracing::info!("{}: K线流订阅成功: {:?}, 开始推送数据", self.base_state.node_id, subscribe_kline_stream_success_response);
                    self.request_id = None;
                    // 修改订阅状态为true
                    self.is_subscribed = true;
                    tracing::warn!("{}: 订阅状态修改为true", self.base_state.node_id);
                }
            }
            ResponseEvent::MarketDataEngine(MarketDataEngineResponse::UnsubscribeKlineStreamSuccess(unsubscribe_kline_stream_success_response)) => {
                if request_id == unsubscribe_kline_stream_success_response.response_id {
                    tracing::info!("{}: K线流取消订阅成功: {:?}, 停止推送数据", self.base_state.node_id, unsubscribe_kline_stream_success_response);
                    self.request_id = None;
                    // 修改订阅状态为false
                    self.is_subscribed = false;
                }
            }   
            _ => {}
        }
    }

    async fn register_exchange(&mut self) -> Result<(), String> {
        let request_id = Uuid::new_v4();
        let register_param = RegisterExchangeParams {
            exchange: self.exchange.clone(),
            sender: self.base_state.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        self.request_id = Some(request_id);
        tracing::warn!("{}: 注册交易所的请求id: {:?}", self.base_state.node_id, self.request_id);

        let command_event = CommandEvent::ExchangeManager(ExchangeManagerCommand::RegisterExchange(register_param));
        tracing::info!("{}注册交易所: {:?}", self.base_state.node_id, command_event);
        if let Err(e) = self.base_state.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %self.base_state.node_id,
                error = ?e,
                "数据源节点发送注册交易所失败"
            );
        }
        Ok(())
        
        
    }

    async fn subscribe_kline_stream(&mut self) -> Result<(), String> {
        let request_id = Uuid::new_v4();
        let params = SubscribeKlineStreamParams {
            strategy_id: self.base_state.strategy_id.clone(),
            node_id: self.base_state.node_id.clone(),
            exchange: self.exchange.clone(),
            symbol: self.symbol.clone(),
            interval: self.interval.clone(),
            frequency: self.frequency.clone(),
            sender: self.base_state.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        self.request_id = Some(request_id);

        let command_event = CommandEvent::MarketDataEngine(MarketDataEngineCommand::SubscribeKlineStream(params));
        tracing::info!("{}订阅k线流: {:?}", self.base_state.node_id, command_event);
        if let Err(e) = self.base_state.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %self.base_state.node_id,
                error = ?e,
                "数据源节点发送数据失败"
            );
        }
        Ok(())
    }

    async fn unsubscribe_kline_stream(&mut self) -> Result<(), String> {
        let request_id = Uuid::new_v4();
        let params = UnsubscribeKlineStreamParams {
            strategy_id: self.base_state.strategy_id.clone(),
            node_id: self.base_state.node_id.clone(),
            exchange: self.exchange.clone(),
            symbol: self.symbol.clone(),
            interval: self.interval.clone(),
            frequency: self.frequency.clone(),
            sender: self.base_state.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        // 设置请求id
        self.request_id = Some(request_id);

        let command_event = CommandEvent::MarketDataEngine(MarketDataEngineCommand::UnsubscribeKlineStream(params));
        if let Err(_) = self.base_state.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %self.base_state.node_id,
                "数据源节点发送数据失败"
            );
        }   
        Ok(())
    }

}

#[derive(Debug)]
pub struct LiveDataNode {
    pub state: Arc<RwLock<Box<dyn NodeState>>>,
    pub from_node_id: Vec<String>, // 来自哪个节点的id
    pub node_type: NodeType,
}

impl Clone for LiveDataNode {
    fn clone(&self) -> Self {
        LiveDataNode { 
            node_type: self.node_type.clone(),
            from_node_id: self.from_node_id.clone(),
            state: self.state.clone(),
        }
    }
}

impl LiveDataNode {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String, 
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        frequency: u32,
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {
        let base_state = BaseNodeState::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            event_publisher,
            vec![market_event_receiver, response_event_receiver],
            LiveDataNodeStateMachine::new(NodeRunState::Created, node_id, node_name),
        );
        Self {
            node_type: NodeType::DataSourceNode, 
            from_node_id: Vec::new(),
            state: Arc::new(RwLock::new(Box::new(LiveDataNodeState {
                base_state,
                exchange, 
                symbol, 
                interval, 
                frequency,
                is_subscribed: false,
                request_id: None
            }))), 
        }
    }

    pub async fn init_node(self) -> Self {
        self.init_node_sender().await
    }

    async fn init_node_sender(self) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node_output_handle = NodeOutputHandle {
            node_id: self.state.read().await.get_node_id().clone(),
            handle_id: "live_data_node_output".to_string(),
            sender: tx,
            connect_count: 0,
        };
        self.state.write().await.get_output_handle_mut().insert("live_data_node_output".to_string(), node_output_handle);
        self
    }

    async fn update_run_state(&mut self, event: NodeStateTransitionEvent) -> Result<(), String> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let node_id = self.state.read().await.get_node_id().clone();
        
        // 获取状态管理器并执行转换
        let (transition_result, run_state_manager) = {
            let mut run_state_manager = self.state.read().await.get_state_machine().clone_box();  // 使用读锁获取当前状态
            let transition_result = run_state_manager.transition(event)?;
            (transition_result, run_state_manager)
        };

        tracing::debug!("{}需要执行的动作: {:?}", node_id, transition_result.get_actions());
        
        
        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(live_data_node_state_action) = action.as_any().downcast_ref::<LiveDataNodeStateAction>() {
                match live_data_node_state_action {
                    LiveDataNodeStateAction::LogTransition => {
                        let current_state = self.state.read().await.base_state.run_state_manager.current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    LiveDataNodeStateAction::LogNodeState => {
                        let current_state = self.state.read().await.base_state.run_state_manager.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    LiveDataNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    LiveDataNodeStateAction::RegisterExchange => {
                        tracing::info!("{}: 注册交易所", node_id);
                        let mut state_guard = self.state.write().await;
                        state_guard.register_exchange().await?;
                    }
                    LiveDataNodeStateAction::SubscribeKline => {
                        let current_state = self.state.read().await.base_state.run_state_manager.current_state();
                        if current_state != NodeRunState::Starting {
                            tracing::warn!(
                                node_id = %node_id,
                                current_state = ?current_state,
                                "节点不在Starting状态, 不订阅K线流"
                            );
                        } else {
                            tracing::info!("{}: 订阅K线流", node_id);
                            let mut state_guard = self.state.write().await;
                            state_guard.subscribe_kline_stream().await?;
                        }
                    }
                    LiveDataNodeStateAction::UnsubscribeKline => {
                        tracing::info!("{}: 取消订阅K线流", node_id);
                        let should_stop = {
                            let node_name = self.state.read().await.base_state.node_name.clone();
                            let current_state = self.state.read().await.base_state.run_state_manager.current_state();
                            if current_state != NodeRunState::Stopping {
                                tracing::warn!(
                                    node_name = %node_name,
                                    current_state = ?current_state,
                                    "节点未运行, 不取消订阅K线流"
                                );
                                false
                            } else {
                                true
                            }
                        }; // 这里读锁被释放
                        if should_stop {
                            let mut state_guard = self.state.write().await;
                            state_guard.unsubscribe_kline_stream().await?;
                        }
                    }
                    _ => {}
                }
            }
            // 动作执行完毕后更新节点最新的状态
            {
                self.state.write().await.base_state.run_state_manager = run_state_manager.clone();
            }

            
            
        }
        

        Ok(())
    }

}

#[async_trait]
impl NodeTrait for LiveDataNode {

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }
    // 获取节点状态
    async fn get_state(&self) -> Arc<RwLock<Box<dyn NodeState<StateMachine = Box<dyn NodeStateMachine>>>>> {
        self.state.clone()
    }

    async fn get_node_id(&self) -> String {
        self.state.read().await.get_node_id().clone()
    }



    async fn get_node_name(&self) -> String {
        self.state.read().await.get_node_name().clone()
    }

    async fn get_message_sender(&self, handle_id: String) -> broadcast::Sender<NodeMessage> {
        self.state.read().await.get_output_handle().get(&handle_id).unwrap().sender.clone()
    }

    async fn get_default_message_sender(&self) -> broadcast::Sender<NodeMessage> {
        self.state.read().await.get_output_handle().get("live_data_node_output").unwrap().sender.clone()
    }

    async fn get_message_receivers(&self) -> Vec<NodeMessageReceiver> {
        self.state.read().await.get_message_receivers().clone()
    }

    async fn get_output_handle(&self) -> HashMap<String, NodeOutputHandle> {
        self.state.read().await.get_output_handle().clone()
    }

    async fn add_message_receiver(&mut self, receiver: NodeMessageReceiver) {
        self.state.write().await.get_message_receivers_mut().push(receiver);
    }

    async fn add_from_node_id(&mut self, from_node_id: String) {
        self.from_node_id.push(from_node_id);
    }

    async fn add_output_handle(&mut self, handle_id: String, sender: broadcast::Sender<NodeMessage>) {
        let node_output_handle = NodeOutputHandle {
            node_id: self.state.read().await.get_node_id().clone(),
            handle_id: handle_id.clone(),
            sender: sender.clone(),
            connect_count: 0,
        };
        self.state.write().await.get_output_handle_mut().insert(handle_id.clone(), node_output_handle.clone());
    }

    async fn add_output_handle_connect_count(&mut self, handle_id: String) {
        self.state.write().await.get_output_handle_mut().get_mut(&handle_id).unwrap().connect_count += 1;
    }


    async fn get_node_run_state(&self) -> NodeRunState {
        self.state.read().await.get_state_machine().current_state()
    }


    // 之后改为setup
    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.state.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.state.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_run_state(NodeStateTransitionEvent::Initialize).await.unwrap();
        tracing::info!("{:?}: 初始化完成", self.state.read().await.get_state_machine().current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_run_state(NodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始启动", state.read().await.get_node_id());
        self.update_run_state(NodeStateTransitionEvent::Start).await.unwrap();

        // 检查是否应该订阅K线流，判断is_subscribed=true
        while !state.read().await.is_subscribed {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        self.update_run_state(NodeStateTransitionEvent::StartComplete).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_run_state(NodeStateTransitionEvent::Stop).await.unwrap();


        // 检查是否应该停止完成，,循环判断is_subscribed=false
        while state.read().await.is_subscribed {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        self.update_run_state(NodeStateTransitionEvent::StopComplete).await?;
        self.cancel_task().await.unwrap();
        Ok(())
    }

    async fn cancel_task(&self) -> Result<(), String> {
        NodeFunction::cancel_task(
            &self.state,
            |state| &state.base_state.cancel_token, 
            |state| &state.base_state.node_id, 
            |state| state.base_state.run_state_manager.current_state()
        ).await;
        Ok(())
    }
}
