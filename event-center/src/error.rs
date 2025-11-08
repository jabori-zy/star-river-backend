use crate::Event;
// use crate::communication::engine::EngineCommand;
use snafu::{Backtrace, Snafu};
use star_river_core::error::ErrorCode;
use star_river_core::error::error_trait::ErrorLanguage;
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum EventCenterError {
    #[snafu(display("Channel [{channel}] not initialized"))]
    ChannelNotInitialized { channel: String, backtrace: Backtrace },

    #[snafu(display("Channel [{channel}] not found"))]
    ChannelNotFound { channel: String, backtrace: Backtrace },

    #[snafu(display("Engine command receiver for [{engine_name}] not found"))]
    EngineCommandReceiverNotFound { engine_name: String, backtrace: Backtrace },

    #[snafu(display("Engine command sender for [{engine_name}] not found"))]
    EngineCommandSenderNotFound { engine_name: String, backtrace: Backtrace },

    #[snafu(transparent)]
    EventSendError {
        source: tokio::sync::broadcast::error::SendError<Event>,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    CommandSendError {
        source: tokio::sync::mpsc::error::SendError<EngineCommand>,
        backtrace: Backtrace,
    },

    #[snafu(display("EventCenter already initialized"))]
    EventCenterInstanceAlreadyInitialized { backtrace: Backtrace },

    #[snafu(display("EventCenter not initialized"))]
    EventCenterInstanceNotInitialized { backtrace: Backtrace },
}

impl star_river_core::error::error_trait::StarRiverErrorTrait for EventCenterError {
    fn get_prefix(&self) -> &'static str {
        "EVENT_CENTER"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            EventCenterError::ChannelNotInitialized { .. } => 1001,
            EventCenterError::ChannelNotFound { .. } => 1002,
            EventCenterError::EventSendError { .. } => 1003,
            EventCenterError::EngineCommandReceiverNotFound { .. } => 1004,
            EventCenterError::EngineCommandSenderNotFound { .. } => 1005,
            EventCenterError::CommandSendError { .. } => 1006,
            EventCenterError::EventCenterInstanceAlreadyInitialized { .. } => 1007,
            EventCenterError::EventCenterInstanceNotInitialized { .. } => 1008,
        };

        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            EventCenterError::ChannelNotInitialized { .. }
                | EventCenterError::ChannelNotFound { .. }
                | EventCenterError::EventSendError { .. }
                | EventCenterError::EngineCommandReceiverNotFound { .. }
                | EventCenterError::EngineCommandSenderNotFound { .. }
                | EventCenterError::CommandSendError { .. }
                | EventCenterError::EventCenterInstanceAlreadyInitialized { .. }
                | EventCenterError::EventCenterInstanceNotInitialized { .. }
        )
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                EventCenterError::ChannelNotInitialized { channel, .. } => {
                    format!("通道 [{}] 未初始化", channel)
                }
                EventCenterError::ChannelNotFound { channel, .. } => {
                    format!("通道 [{}] 未找到", channel)
                }
                EventCenterError::EngineCommandReceiverNotFound { engine_name, .. } => {
                    format!("引擎 [{}] 的命令接收器未找到", engine_name)
                }
                EventCenterError::EngineCommandSenderNotFound { engine_name, .. } => {
                    format!("引擎 [{}] 的命令发送器未找到", engine_name)
                }
                EventCenterError::EventSendError { source, .. } => {
                    format!("事件发送错误: {}", source)
                }
                EventCenterError::CommandSendError { source, .. } => {
                    format!("命令发送错误: {}", source)
                }
                EventCenterError::EventCenterInstanceAlreadyInitialized { .. } => "事件中心实例已初始化".to_string(),
                EventCenterError::EventCenterInstanceNotInitialized { .. } => "事件中心实例未初始化".to_string(),
            },
        }
    }
}
