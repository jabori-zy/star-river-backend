use snafu::{Backtrace, Snafu};
use star_river_core::error::{
    ErrorCode,
    error_trait::{ErrorLanguage, StarRiverErrorTrait},
};

/// 泛型事件中心错误类型
///
/// 泛型参数：
/// - `E`: Event 类型（用于 EventSendError）
/// - `C`: Command 类型（用于 CommandSendError）
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum EventCenterError {
    #[snafu(display("Channel [{channel}] not initialized"))]
    ChannelNotInitialized { channel: String, backtrace: Backtrace },

    #[snafu(display("Channel [{channel}] not found"))]
    ChannelNotFound { channel: String, backtrace: Backtrace },

    #[snafu(display("Engine command receiver for [{target}] not found"))]
    CommandReceiverNotFound { target: String, backtrace: Backtrace },

    #[snafu(display("Engine command sender for [{target}] not found"))]
    CommandSenderNotFound { target: String, backtrace: Backtrace },

    #[snafu(display("EventCenter already initialized"))]
    InstanceAlreadyInit { backtrace: Backtrace },

    #[snafu(display("EventCenter not initialized"))]
    InstanceNotInit { backtrace: Backtrace },
}

impl StarRiverErrorTrait for EventCenterError {
    fn get_prefix(&self) -> &'static str {
        "EVENT_CENTER"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            EventCenterError::ChannelNotInitialized { .. } => 1001,
            EventCenterError::ChannelNotFound { .. } => 1002,
            EventCenterError::CommandReceiverNotFound { .. } => 1003,
            EventCenterError::CommandSenderNotFound { .. } => 1004,
            EventCenterError::InstanceAlreadyInit { .. } => 1005,
            EventCenterError::InstanceNotInit { .. } => 1006,
        };

        format!("{}_{:04}", prefix, code)
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
                EventCenterError::CommandReceiverNotFound { target, .. } => {
                    format!("命令目标 [{}] 的命令接收器未找到", target)
                }
                EventCenterError::CommandSenderNotFound { target, .. } => {
                    format!("命令目标 [{}] 的命令发送器未找到", target)
                }
                EventCenterError::InstanceAlreadyInit { .. } => "事件中心实例已初始化".to_string(),
                EventCenterError::InstanceNotInit { .. } => "事件中心实例未初始化".to_string(),
            },
        }
    }
}
