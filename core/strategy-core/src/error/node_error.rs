use std::sync::Arc;

use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::{HandleId, NodeName},
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode, generate_error_code_chain},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum NodeError {
    #[snafu(display("unsupported backtest node type: {node_type}"))]
    UnsupportedNodeType { node_type: String, backtrace: Backtrace },

    #[snafu(display("config field value is null: {field_name}"))]
    ConfigFieldValueNull { field_name: String, backtrace: Backtrace },

    #[snafu(display("@[{node_name}] config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed {
        node_name: String,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("kline node [{node_id}] name is null"))]
    NodeNameIsNull { node_id: String, backtrace: Backtrace },

    #[snafu(display("kline node id is null"))]
    NodeIdIsNull { backtrace: Backtrace },

    #[snafu(display("kline node [{node_id}] data is null"))]
    NodeDataIsNull { node_id: String, backtrace: Backtrace },

    #[snafu(display("@[{node_name}] config '{config_name}' should be greater than or equal to(>= 0) zero, but got {config_value}"))]
    ValueNotGreaterThanOrEqualToZero {
        node_name: String,
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },

    // > 0
    #[snafu(display("@[{node_name}] config '{config_name}' should be greater than(> 0) zero, but got {config_value}"))]
    ValueNotGreaterThanZero {
        node_name: String,
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] mount node cycle tracker failed"))]
    NodeCycleTrackerMountFailed { node_name: String, backtrace: Backtrace },

    #[snafu(display("@[{node_name}] output handle not found: {handle_id}"))]
    OutputHandleNotFound {
        node_name: NodeName,
        handle_id: HandleId,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] node event send failed: {handle_id}, reason: {source}"))]
    NodeEventSendFailed {
        node_name: NodeName,
        handle_id: HandleId,
        #[snafu(source(true))]
        source: Arc<dyn std::error::Error + Send + Sync + 'static>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] strategy command send failed, reason: {source}"))]
    StrategyCommandSendFailed {
        node_name: NodeName,
        #[snafu(source(true))]
        source: Arc<dyn std::error::Error + Send + Sync + 'static>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] node command send failed, reason: {source}"))]
    NodeCommandSendFailed {
        node_name: NodeName,
        #[snafu(source(true))]
        source: Arc<dyn std::error::Error + Send + Sync + 'static>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] strategy command response receive failed, reason: {source}"))]
    StrategyCmdRespRecvFailed {
        node_name: NodeName,
        #[snafu(source(true))]
        source: tokio::sync::oneshot::error::RecvError,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] strategy error: {source}"))]
    StrategyError {
        node_name: NodeName,
        source: Arc<dyn StarRiverErrorTrait + Send + Sync + 'static>,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for NodeError
impl StarRiverErrorTrait for NodeError {
    fn get_prefix(&self) -> &'static str {
        "NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            NodeError::UnsupportedNodeType { .. } => 1001,              // unsupported node type
            NodeError::ConfigFieldValueNull { .. } => 1002,             // node config field value is null
            NodeError::ConfigDeserializationFailed { .. } => 1003,      // node config deserialization failed
            NodeError::NodeNameIsNull { .. } => 1004,                   // node name is null
            NodeError::NodeIdIsNull { .. } => 1005,                     // node id is null
            NodeError::NodeDataIsNull { .. } => 1006,                   // node data is null
            NodeError::ValueNotGreaterThanOrEqualToZero { .. } => 1007, // value not greater than or equal to zero (>= 0)
            NodeError::ValueNotGreaterThanZero { .. } => 1008,          // value not greater than zero (> 0)
            NodeError::NodeCycleTrackerMountFailed { .. } => 1009,      // node cycle tracker mount failed
            NodeError::OutputHandleNotFound { .. } => 1010,             // output handle not found
            NodeError::NodeEventSendFailed { .. } => 1011,              // node event send failed
            NodeError::StrategyCommandSendFailed { .. } => 1012,        // strategy command send failed
            NodeError::NodeCommandSendFailed { .. } => 1013,            // node command send failed.
            NodeError::StrategyCmdRespRecvFailed { .. } => 1014,        // strategy command response receive failed.
            NodeError::StrategyError { .. } => 1015,                    // strategy error.
        };
        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            NodeError::UnsupportedNodeType { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            NodeError::ConfigFieldValueNull { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            NodeError::ConfigDeserializationFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            NodeError::NodeNameIsNull { .. } => StatusCode::BAD_REQUEST,
            NodeError::NodeIdIsNull { .. } => StatusCode::BAD_REQUEST,
            NodeError::NodeDataIsNull { .. } => StatusCode::BAD_REQUEST,
            NodeError::ValueNotGreaterThanOrEqualToZero { .. } => StatusCode::BAD_REQUEST,
            NodeError::ValueNotGreaterThanZero { .. } => StatusCode::BAD_REQUEST,
            NodeError::NodeCycleTrackerMountFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            NodeError::OutputHandleNotFound { .. } => StatusCode::BAD_REQUEST,
            NodeError::NodeEventSendFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            NodeError::StrategyCommandSendFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            NodeError::NodeCommandSendFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            NodeError::StrategyCmdRespRecvFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            NodeError::StrategyError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => {
                match self {
                    // non-transparent errors - use custom message
                    NodeError::UnsupportedNodeType { node_type, .. } => {
                        format!("不支持的节点类型: {}", node_type)
                    }
                    NodeError::ConfigFieldValueNull { field_name, .. } => {
                        format!("节点配置字段值为空: {}", field_name)
                    }
                    NodeError::ConfigDeserializationFailed { node_name, source, .. } => {
                        format!("@[{node_name}] 配置反序列化失败，原因: {}", source)
                    }
                    NodeError::NodeNameIsNull { node_id, .. } => {
                        format!("节点 [{node_id}] 名称是空")
                    }
                    NodeError::NodeIdIsNull { .. } => {
                        format!("节点 id 是空")
                    }
                    NodeError::NodeDataIsNull { node_id, .. } => {
                        format!("节点 [{node_id}] 数据是空")
                    }
                    NodeError::ValueNotGreaterThanOrEqualToZero {
                        node_name,
                        config_name,
                        config_value,
                        ..
                    } => {
                        format!("@[{node_name}] 配置 '{config_name}' 应该大于等于零(>= 0)，但值为 {config_value}")
                    }
                    NodeError::ValueNotGreaterThanZero {
                        node_name,
                        config_name,
                        config_value,
                        ..
                    } => {
                        format!("[{node_name}] 配置 {config_name} 应该大于零(> 0)，但值为 {config_value}")
                    }

                    NodeError::NodeCycleTrackerMountFailed { node_name, .. } => {
                        format!("[{node_name}] 挂载节点周期追踪器失败")
                    }
                    NodeError::OutputHandleNotFound { handle_id, .. } => {
                        format!("输出句柄未找到: {}", handle_id)
                    }
                    NodeError::NodeEventSendFailed { handle_id, source, .. } => {
                        format!("节点事件发送失败: {}, 原因: {}", handle_id, source)
                    }
                    NodeError::StrategyCommandSendFailed { node_name, source, .. } => {
                        format!("@[{node_name}] 策略命令发送失败, 原因: {}", source)
                    }
                    NodeError::NodeCommandSendFailed { node_name, source, .. } => {
                        format!("@[{node_name}] 节点命令发送失败, 原因: {}", source)
                    }
                    NodeError::StrategyCmdRespRecvFailed { node_name, source, .. } => {
                        format!("@[{node_name}] 策略命令响应接收失败, 原因: {}", source)
                    }
                    NodeError::StrategyError { node_name, source, .. } => {
                        format!("@[{node_name}] 策略错误: {}", source)
                    }
                }
            }
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // non-transparent errors - return own error code
            NodeError::StrategyError { source, .. } => generate_error_code_chain(source.as_ref()),
            _ => vec![self.error_code()],
        }
    }
}
