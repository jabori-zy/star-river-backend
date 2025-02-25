use heartbeat::Heartbeat;

#[derive(Clone, Debug)]
pub struct AppState {
    pub heartbeat: Heartbeat,
}

impl AppState {
    pub fn new() -> Self {
        Self { heartbeat: Heartbeat::new(1000) }
    }
}



