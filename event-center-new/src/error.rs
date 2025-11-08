use crate::Event;
use crate::EngineCommand;
use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode,ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain};
use event_center_core::error::EventCenterError as EventCenterCommonError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum EventCenterErrorBasic {

    #[snafu(transparent)]
    EventCenterCommonError { source: EventCenterCommonError, backtrace: Backtrace },

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

impl StarRiverErrorTrait for EventCenterErrorBasic {
    fn get_prefix(&self) -> &'static str {
        "EVENT_CENTER"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            EventCenterErrorBasic::EventCenterCommonError { .. } => 1001,
            EventCenterErrorBasic::EventSendFailed { .. } => 1002,
            EventCenterErrorBasic::CommandSendFailed { .. } => 1003,
        };

        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            EventCenterErrorBasic::EventCenterCommonError { source, .. } => generate_error_code_chain(source),
            EventCenterErrorBasic::EventSendFailed { .. } => vec![self.error_code()],
            EventCenterErrorBasic::CommandSendFailed { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                EventCenterErrorBasic::EventCenterCommonError { source, .. } => source.error_message(language),
                EventCenterErrorBasic::EventSendFailed { source, .. } => {
                    format!("事件发送失败: {}", source)
                }
                EventCenterErrorBasic::CommandSendFailed { source, .. } => {
                    format!("命令发送失败: {}", source)
                }
            },
        }
    }
}
