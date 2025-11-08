use chrono::{DateTime, Utc};
use std::fmt::Debug;
use std::ops::Deref;
use tokio::sync::oneshot;

use super::response::Response;

// ================================ Engine Command Base ================================
#[derive(Debug)]
pub struct CommandBase<S> {
    pub sender: String,
    pub datetime: DateTime<Utc>,
    pub responder: oneshot::Sender<Response<S>>,
}

#[derive(Debug)]
pub struct Command<T, S> {
    pub command_base: CommandBase<S>,
    pub command_payload: T,
}

impl<T, S> Command<T, S> {
    pub fn new(sender: String, responder: oneshot::Sender<Response<S>>, command_payload: T) -> Self {
        let command_base = CommandBase {
            sender,
            datetime: Utc::now(),
            responder,
        };
        Self {
            command_base,
            command_payload
        }
    }

    pub fn sender(&self) -> String {
        self.command_base.sender.clone()
    }
}

impl<T, S> Command<T,S> {

    pub fn datetime(&self) -> DateTime<Utc> {
        self.command_base.datetime
    }

    pub fn respond(self, response: Response<S>) {
        let _ = self.command_base.responder.send(response);
    }
}

impl<T, S> Deref for Command<T, S> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self
            .command_payload
    }
}