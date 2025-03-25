mod state_manager;

use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use tokio::sync::broadcast;
use utils::get_utc8_timestamp;
use crate::NodeSender;
use crate::NodeReceiver;
use crate::NodeOutputHandle;
use crate::NodeRunState;
use crate::node::NodeTrait;
use serde::{Serialize, Deserialize};
use strum::EnumString;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use types::strategy::message::{NodeMessage, IndicatorMessage, SignalMessage, Signal};
use event_center::EventPublisher;
use event_center::strategy_event::StrategyEvent;
use event_center::Event;
use strum_macros::Display;
use tokio_util::sync::CancellationToken;
use crate::node::if_else_node::state_manager::IfElseNodeStateManager;
use crate::node::NodeStateTransitionEvent;
use std::time::Duration;
use crate::node::if_else_node::state_manager::IfElseNodeStateAction;


#[derive(Debug)]
pub struct IfElseNodeState {
    pub node_id: String,
    pub node_name: String,
    pub current_batch_id: Option<String>,
    pub is_processing: bool,
    pub received_flag: HashMap<String, bool>, // 用于记录每个节点的数据是否接收完成
    pub received_value: HashMap<String, Option<NodeMessage>>, // 用于记录每个节点的数据
    pub cases: Vec<Case>,
    pub node_output_handle: HashMap<String, NodeSender>, // 节点的出口 {handle_id: sender}, 每个handle对应一个sender
    pub node_output_handle1: HashMap<String, NodeOutputHandle>, // 节点的出口连接数 {handle_id: count}, 每个handle对应一个连接数
    pub event_publisher: EventPublisher, // 事件发布器
    pub enable_event_publish: bool, // 是否启用事件发布
    pub run_state_manager: IfElseNodeStateManager,
    pub cancel_token: CancellationToken,
    pub receivers: Vec<NodeReceiver>,
    pub from_node_id: Vec<String>,
}

// 条件分支节点
#[derive(Debug, Clone)]
pub struct IfElseNode {
    
    pub sender: NodeSender,
    pub node_case_sender: HashMap<String, NodeSender>,
    
    pub state: Arc<RwLock<IfElseNodeState>>,
}





impl IfElseNode {

    pub fn new(
        node_id: String, 
        node_name: String, 
        cases: Vec<Case>, 
        event_publisher: EventPublisher,
    ) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        Self { 
            sender: NodeSender::new(node_id.clone(), "if_else".to_string(), tx), 
            node_case_sender: HashMap::new(),
            state: Arc::new(RwLock::new(IfElseNodeState {
                node_id: node_id.clone(),
                node_name: node_name.clone(),
                current_batch_id: None,
                is_processing: false,
                received_flag: HashMap::new(),
                received_value: HashMap::new(),
                cases,
                node_output_handle: HashMap::new(),
                node_output_handle1: HashMap::new(),
                event_publisher,
                enable_event_publish: false,
                run_state_manager: IfElseNodeStateManager::new(NodeRunState::Created, node_id, node_name),
                cancel_token: CancellationToken::new(),
                receivers: Vec::new(),
                from_node_id: Vec::new(),
            })),
        }
    }

    // 初始化接收标记
    async fn init_received_flag(state: Arc<RwLock<IfElseNodeState>>) {
        let mut state_guard = state.write().await;
        for from_node_id in state_guard.from_node_id.clone() {
            state_guard.received_flag.insert(from_node_id, false);
        }
    }

    async fn init_received_value(state: Arc<RwLock<IfElseNodeState>>) {
        let mut state_guard = state.write().await;
        for from_node_id in state_guard.from_node_id.clone() {
            state_guard.received_value.insert(from_node_id, None);
        }
    }

    pub async fn init_node(self) -> Self {
        self.init_node_sender().await
    }

    async fn init_node_sender(self) -> Self {
        let mut handles = HashMap::new();
        let state = self.state.clone();
        let node_id = state.read().await.node_id.clone();
        let cases = state.read().await.cases.clone();
        
        for case in cases {
            let (tx, _) = broadcast::channel::<NodeMessage>(100);
            let case_sender = NodeSender::new(node_id.clone(), format!("if_else_node_case_{}_output", case.case_id), tx);
            handles.insert(format!("if_else_node_case_{}_output", case.case_id), case_sender);
        }

        // 添加else handle
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let else_sender = NodeSender::new(node_id.clone(), "if_else_node_else_output".to_string(), tx);
        handles.insert("if_else_node_else_output".to_string(), else_sender);

        self.state.write().await.node_output_handle = handles;
        self
    }

    // 获取默认的handle
    pub async fn get_default_handle(state: &Arc<RwLock<IfElseNodeState>>) -> NodeSender {
        let state = state.read().await;
        // 默认节点是else handle
        state.node_output_handle.get("if_else_node_else_output").unwrap().clone()
    }

    pub async fn listen_message(state: Arc<RwLock<IfElseNodeState>>) {
        let streams: Vec<_> = state.read().await.receivers.iter()
            .map(|receiver| BroadcastStream::new(receiver.get_receiver()))
            .collect();
        let mut combined_stream = select_all(streams);

        let state_clone = state.clone();
        let cancel_token = state.read().await.cancel_token.clone();
        let node_id = state.read().await.node_id.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点消息监听进程已中止", node_id);
                        break;
                    }
                    result = combined_stream.next() => {
                        match result {
                            Some(Ok(message)) => {
                                match message {
                                    NodeMessage::Indicator(indicator_message) => {
                                        // tracing::debug!("条件节点收到数据: message={:?}", indicator_message);
                                        Self::handle_indicator_message(state_clone.clone(), indicator_message).await;
                                    }
                                    _ => {}
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("节点 {} 接收消息错误: {}", node_id, e);
                            }
                            None => {
                                tracing::info!("节点 {} 所有消息流已关闭", node_id);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    // 处理指标消息
    async fn handle_indicator_message(state: Arc<RwLock<IfElseNodeState>>, indicator_message: IndicatorMessage) {
        let from_node_id = indicator_message.from_node_id.clone();
        let mut state_guard = state.write().await;

        // 更新接收值
        let received_value = state_guard.received_value.get_mut(&from_node_id).unwrap();
        *received_value = Some(NodeMessage::Indicator(indicator_message));

        // 更新接收标记
        let received_flag = state_guard.received_flag.get_mut(&from_node_id).unwrap();
        *received_flag = true;
    }

    async fn cancel_task(state: Arc<RwLock<IfElseNodeState>>) {
        let state_guard = state.read().await;
        state_guard.cancel_token.cancel();
        tracing::info!("{}: 节点已安全停止, 当前节点状态: {:?}", state_guard.node_id, state_guard.run_state_manager.current_state());
    }

    async fn update_run_state(state: Arc<RwLock<IfElseNodeState>>, event: NodeStateTransitionEvent) -> Result<(), String> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let node_id = state.read().await.node_id.clone();
        
        // 获取状态管理器并执行转换
        let (transition_result, state_manager) = {
            let node_guard = state.read().await;  // 使用读锁获取当前状态
            let mut state_manager = node_guard.run_state_manager.clone();
            let transition_result = state_manager.transition(event)?;
            (transition_result, state_manager)
        };

        tracing::info!("{}需要执行的动作: {:?}", node_id, transition_result.actions);
        // 执行转换后需要执行的动作
        for action in transition_result.actions.clone() {  // 克隆actions避免移动问题
            match action {
                IfElseNodeStateAction::LogTransition => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.new_state);
                }
                IfElseNodeStateAction::LogNodeState => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                }

                IfElseNodeStateAction::ListenAndHandleMessage => {
                    tracing::info!("{}: 开始监听节点传递的message", node_id);
                    Self::listen_message(state.clone()).await;
                }
                IfElseNodeStateAction::InitReceivedFlag => {
                    tracing::info!("{}: 开始初始化接收标记", node_id);
                    Self::init_received_flag(state.clone()).await;
                }
                IfElseNodeStateAction::InitReceivedValue => {
                    tracing::info!("{}: 开始初始化接收值", node_id);
                    Self::init_received_value(state.clone()).await;
                }
                IfElseNodeStateAction::Evaluate => {
                    tracing::info!("{}: 开始判断条件", node_id);
                    Self::evaluate(state.clone()).await;
                }
                _ => {}
                // 所有动作执行完毕后更新节点最新的状态
                
            }
            // 所有动作执行完毕后更新节点最新的状态
            {
                let mut node_guard = state.write().await;
                node_guard.run_state_manager = state_manager.clone();
            }
        }
                    
        Ok(())
    }
        


    // 开始评估各个分支
    async fn evaluate(state: Arc<RwLock<IfElseNodeState>>) {
        let cancel_token = state.read().await.cancel_token.clone();
        let node_id = state.read().await.node_id.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点条件判断进程已中止", node_id);
                        break;
                    }
                    _ = async {
                        // 检查状态并获取需要的数据
                        let should_evaluate = {
                            let mut state_guard = state.write().await;
                            // 如果所有节点都已接收数据，则返回true
                            if !state_guard.received_flag.values().all(|flag| *flag) {
                                false
                            } else {
                                // 重置标记位
                                for flag in state_guard.received_flag.values_mut() {
                                    *flag = false;
                                }
                                true
                            }
                        };

                        if should_evaluate {
                            // 获取需要评估的 cases（克隆数据以释放锁）
                            let cases = {
                                let state_guard = state.read().await;
                                let mut cases = state_guard.cases.clone();
                                cases.sort_by_key(|case| case.case_id);
                                cases
                            };

                            // 在锁外执行评估
                            let mut case_matched = false;
                            for case in cases {
                                let case_result = Self::evaluate_case(case.clone(), state.clone()).await;
                                // 如果为true，则发送消息到分支, 并且后续的case不进行评估
                                if case_result {
                                    let state_guard = state.read().await;
                                    let case_sender = state_guard.node_output_handle.get(&format!("if_else_node_case_{}_output", case.case_id)).unwrap();
                                    let signal_message = SignalMessage {
                                        from_node_id: state_guard.node_id.clone(),
                                        from_node_name: state_guard.node_name.clone(),
                                        from_node_handle: format!("if_else_node_case_{}_output", case.case_id),
                                        signal: Signal::True,
                                        message_timestamp: get_utc8_timestamp()
                                    };
                                    tracing::warn!("条件节点发送信号: {:?}", signal_message);
                                    if let Err(e) = case_sender.send(NodeMessage::Signal(signal_message.clone())) {
                                        tracing::error!("节点 {} 发送信号失败: {}", state_guard.node_id, e);
                                    }

                                    // 发送事件
                                    if state_guard.enable_event_publish {
                                        let event = Event::Strategy(StrategyEvent::NodeMessage(NodeMessage::Signal(signal_message)));
                                        if let Err(e) = state_guard.event_publisher.publish(event.into()) {
                                            tracing::error!(
                                                node_id = %state_guard.node_id,
                                                "条件节点发送信号事件失败"
                                            );
                                        }
                                    }
                                    case_matched = true;
                                    break;
                                }
                            }

                            // 只有当所有case都为false时才执行else
                            if !case_matched {
                                let state_guard = state.read().await;
                                let else_sender = state_guard.node_output_handle.get("if_else_node_else_output").unwrap();
                                let signal_message = SignalMessage {
                                    from_node_id: state_guard.node_id.clone(),
                                    from_node_name: state_guard.node_name.clone(),
                                    from_node_handle: "if_else_node_else_output".to_string(),
                                    signal: Signal::False,
                                    message_timestamp: get_utc8_timestamp()
                                };
                                // tracing::warn!("条件节点发送信号: {:?}", signal_message);
                                if let Err(e) = else_sender.send(NodeMessage::Signal(signal_message.clone())) {
                                    tracing::error!("节点 {} 发送信号失败: {}", state_guard.node_id, e);
                                }

                                // 发送事件
                                if state_guard.enable_event_publish {
                                    let event = Event::Strategy(StrategyEvent::NodeMessage(NodeMessage::Signal(signal_message)));
                                    if let Err(e) = state_guard.event_publisher.publish(event.into()) {
                                        tracing::error!(
                                            node_id = %state_guard.node_id,
                                            "条件节点发送信号事件失败"
                                        );
                                    }
                                }
                            }
                        }

                        // 添加短暂延迟避免过度循环
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    } => {}
                }
            }
        });
    }

    async fn evaluate_case(case: Case, state: Arc<RwLock<IfElseNodeState>>) -> bool {
        let logic_operator = case.logic_operator;
        match logic_operator {
            LogicOperator::And => {
                Self::evaluate_and_conditions(case.conditions, state.clone()).await
            }
            LogicOperator::Or => {
                Self::evaluate_or_conditions(case.conditions, state.clone()).await
            }
        }
    }

    fn get_variable_value(
        node_id: &str, 
        variable_name: &str, 
        received_value: &HashMap<String, Option<NodeMessage>>
    ) -> Option<f64> {
        received_value
            .get(node_id)?
            .as_ref()?
            .as_indicator()?
            .indicator_data
            .get_latest_indicator_value()
            .get(variable_name)
            .map(|v| v.value)
    }

    // 评估and条件组
    async fn evaluate_and_conditions(conditions: Vec<Condition>, state: Arc<RwLock<IfElseNodeState>>) -> bool{
        let state = state.read().await;
        let received_value = &state.received_value;
        let mut result = true;
        for condition in conditions {
            // 获取左值
            let left_node_id = condition.left_variable.node_id.unwrap();
            let left_variabale = condition.left_variable.variable;
            let left_value = Self::get_variable_value(&left_node_id, &left_variabale, received_value);

            // 获取右值
            let right_var_type = condition.right_variable.var_type;
            let right_value = match right_var_type {
                VarType::Variable => {
                    let right_node_id = condition.right_variable.node_id.unwrap();
                    let right_variabale = condition.right_variable.variable;
                    Self::get_variable_value(&right_node_id, &right_variabale, received_value)
                },
                VarType::Constant => {
                    let right_variabale = condition.right_variable.variable;
                    Some(right_variabale.parse::<f64>().unwrap())
                },
            };

            let operator = condition.comparison_operator;
            
            if left_value.is_some() && right_value.is_some() {
                let left_value = left_value.unwrap();
                let right_value = right_value.unwrap();
                let condition_result = match operator {
                    ComparisonOperator::GreaterThan => left_value > right_value,
                    ComparisonOperator::LessThan => left_value < right_value,
                    ComparisonOperator::Equal => (left_value - right_value).abs() < f64::EPSILON,
                    ComparisonOperator::GreaterThanOrEqual => left_value >= right_value,
                    ComparisonOperator::LessThanOrEqual => left_value <= right_value,
                    ComparisonOperator::NotEqual => left_value != right_value,
                };
                // tracing::warn!("左值: {:?}, 比较符号: {:?}, 右值: {:?}, 结果: {:?}", left_value, operator.to_string(), right_value, condition_result);
                
                // 如果当前条件为真，则将结果设置为真
                result = result && condition_result;
            } 
        }
        result

        

        

    }

    async fn evaluate_or_conditions(conditions: Vec<Condition>, state: Arc<RwLock<IfElseNodeState>>) -> bool {
        false
    }



}


#[async_trait]
impl NodeTrait for IfElseNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }


    async fn get_node_name(&self) -> String {
        self.state.read().await.node_name.clone()
    }

    async fn get_node_id(&self) -> String {
        self.state.read().await.node_id.clone()
    }

    async fn get_node_sender(&self, handle_id: String) -> NodeSender {
        self.state.read().await.node_output_handle.get(&handle_id).unwrap().clone()
    }

    async fn get_default_node_sender(&self) -> NodeSender {
        self.state.read().await.node_output_handle.get("if_else_node_else_output").unwrap().clone()
    }

    async fn add_message_receiver(&mut self, receiver: NodeReceiver) {  
        self.state.write().await.receivers.push(receiver);
    }

    async fn add_from_node_id(&mut self, from_node_id: String) {
        self.state.write().await.from_node_id.push(from_node_id);
    }

    async fn add_node_output_handle(&mut self, handle_id: String, sender: NodeSender) {
        self.state.write().await.node_output_handle.insert(handle_id.clone(), sender.clone());
        self.state.write().await.node_output_handle1.insert(handle_id.clone(), NodeOutputHandle {
            handle_id: handle_id.clone(),
            sender: sender.clone(),
            connect_count: 0,
        });
    }

    async fn add_node_output_handle_connect_count(&mut self, handle_id: String) {
        self.state.write().await.node_output_handle1.get_mut(&handle_id).unwrap().connect_count += 1;
    }

    async fn enable_node_event_push(&mut self) {
        self.state.write().await.enable_event_publish = true;
        tracing::info!("{}: 节点事件推送已启用", self.state.read().await.node_name);
    }

    async fn disable_node_event_push(&mut self) {
        self.state.write().await.enable_event_publish = false;
        tracing::info!("{}: 节点事件推送已禁用", self.state.read().await.node_name);
    }


    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.state.read().await.node_name);
        tracing::info!("{}: 开始初始化", self.state.read().await.node_name);
        // 开始初始化 created -> Initialize
        Self::update_run_state(self.state.clone(), NodeStateTransitionEvent::Initialize).await.unwrap();

        tracing::info!("{:?}: 初始化完成", self.state.read().await.run_state_manager.current_state());
        // 初始化完成 Initialize -> InitializeComplete
        Self::update_run_state(self.state.clone(), NodeStateTransitionEvent::InitializeComplete).await?;


        Ok(())
        
    }

    async fn start(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始启动", state.read().await.node_id);
        // 切换为starting状态
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::Start).await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 切换为running状态
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::StartComplete).await.unwrap();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始停止", state.read().await.node_id);
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::Stop).await.unwrap();
        // 等待所有任务结束
        Self::cancel_task(state.clone()).await;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }

    async fn get_node_run_state(&self) -> NodeRunState {
        self.state.read().await.run_state_manager.current_state()
    }
}


// 比较操作符
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
pub enum ComparisonOperator {
    #[serde(rename = ">")]
    #[strum(serialize = ">")]
    GreaterThan,
    #[serde(rename = "<")]
    #[strum(serialize = "<")]
    LessThan,
    #[serde(rename = "=")]
    #[strum(serialize = "=")]
    Equal,
    #[serde(rename = "!=")]
    #[strum(serialize = "!=")]
    NotEqual,
    #[serde(rename = ">=")]
    #[strum(serialize = ">=")]
    GreaterThanOrEqual,
    #[serde(rename = "<=")]
    #[strum(serialize = "<=")]
    LessThanOrEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VarType {
    #[serde(rename = "variable")]
    Variable,
    #[serde(rename = "constant")]
    Constant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    #[serde(rename = "nodeId")]
    pub node_id: Option<String>,
    #[serde(rename = "varType")]
    pub var_type: VarType,
    #[serde(rename = "varibale")]  // 注意：保持与 JSON 中的拼写一致
    pub variable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    #[serde(rename = "comparisonOperator")]
    pub comparison_operator: ComparisonOperator,
    #[serde(rename = "leftVariable")]
    pub left_variable: Variable,
    #[serde(rename = "rightVariable")]
    pub right_variable: Variable,
}

// 逻辑操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicOperator {
    #[serde(rename = "and")]
    And,
    #[serde(rename = "or")]
    Or,
}

// 分支
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    #[serde(rename = "caseId")]
    pub case_id: i32,
    pub conditions: Vec<Condition>,
    #[serde(rename = "logicalOperator")]
    pub logic_operator: LogicOperator,
}

// 使用示例
impl Case {
    pub fn from_json(json_str: &str) -> Result<Case, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}