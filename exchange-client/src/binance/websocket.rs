use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
// Represents a subscribed stream
pub struct Stream {
    pub stream_name: String,
}

impl Stream {
    pub fn new(stream_name: &str) -> Self {
        Self {
            stream_name: stream_name.to_owned(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.stream_name
    }
}

impl Display for Stream {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.stream_name)
    }
}
