use crate::constants::MAX_MESSAGE_SIZE;
use bincode::Options;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum Request {
    #[default]
    Noop,
    Connect {
        address: String,
        port: String,
    },
    Disconnect,
    Query,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Response {
    Ok,
    Error(String),
}

impl Request {
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::DefaultOptions::new()
            .with_limit(MAX_MESSAGE_SIZE as u64)
            .serialize(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::DefaultOptions::new()
            .with_limit(MAX_MESSAGE_SIZE as u64)
            .allow_trailing_bytes()
            .deserialize(bytes)
    }
}

impl Response {
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::DefaultOptions::new()
            .with_limit(MAX_MESSAGE_SIZE as u64)
            .serialize(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::DefaultOptions::new()
            .with_limit(MAX_MESSAGE_SIZE as u64)
            .allow_trailing_bytes()
            .deserialize(bytes)
    }
}
