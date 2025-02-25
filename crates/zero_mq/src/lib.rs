use zeromq::{PubSocket, Socket, SocketSend};
use serde::{Serialize, Deserialize};
use strum::Display;


#[derive(Debug, Display, Clone, Serialize, Deserialize)]
pub enum Topic {
    Market,
    Trade,
    Order,
    Position,
    Indicator,
    Command,
}


pub struct EventCenter {
    socket: PubSocket,
    endpoint: String,

}

impl EventCenter {
    pub fn new(endpoint: &str) -> Self {
        let socket = PubSocket::new();
        Self { socket, endpoint: endpoint.to_string() }
    }

    async fn bind(&mut self) -> Result<(), String> {
        self.socket.bind(self.endpoint.as_str()).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn publish(&mut self, topic: Topic, message: String) -> Result<(), String> {
        let message = format!("{}: {}", topic, message);
        self.socket.send(message.into()).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}