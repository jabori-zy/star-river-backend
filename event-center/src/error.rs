use event_center_core::error::EventCenterError as EventCenterCommonError;
use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain};

use crate::{EngineCommand, Event};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum EventCenterError {
    #[snafu(transparent)]
    EventCenterCommonError {
        source: EventCenterCommonError,
        backtrace: Backtrace,
    },

    #[snafu(display("Event send failed: {source}"))]
    EventSendFailed {
        source: tokio::sync::broadcast::error::SendError<Event>,
        backtrace: Backtrace,
    },

    #[snafu(display("Command send failed: {source}"))]
    CommandSendFailed {
        source: tokio::sync::mpsc::error::SendError<EngineCommand>,
        backtrace: Backtrace,
    },
}

impl StarRiverErrorTrait for EventCenterError {
    fn get_prefix(&self) -> &'static str {
        "EVENT_CENTER"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            EventCenterError::EventCenterCommonError { .. } => 1001,
            EventCenterError::EventSendFailed { .. } => 1002,
            EventCenterError::CommandSendFailed { .. } => 1003,
        };

        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            EventCenterError::EventCenterCommonError { source, .. } => generate_error_code_chain(source, self.error_code()),
            EventCenterError::EventSendFailed { .. } => vec![self.error_code()],
            EventCenterError::CommandSendFailed { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                EventCenterError::EventCenterCommonError { source, .. } => source.error_message(language),
                EventCenterError::EventSendFailed { source, .. } => {
                    format!("事件发送失败: {}", source)
                }
                EventCenterError::CommandSendFailed { source, .. } => {
                    format!("命令发送失败: {}", source)
                }
            },
        }
    }
}
