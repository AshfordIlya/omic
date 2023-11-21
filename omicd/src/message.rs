use std::{
    os::fd::{AsRawFd, RawFd},
    sync::Mutex,
};

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
    Status,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Response {
    Ok,
    Error(String),
    Connection(Connection),
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

pub struct State {
    pub socket: uds::UnixSeqpacketListener,
    pub connection: Mutex<Connection>,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Connection {
    pub status: Status,
    pub address: Option<String>,
    pub port: Option<String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Status {
    Connected,
    Disconnected,
    Error,
}

impl AsRawFd for State {
    fn as_raw_fd(&self) -> RawFd {
        self.socket.as_raw_fd()
    }
}

impl Connection {
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::DefaultOptions::new()
            .with_limit(MAX_MESSAGE_SIZE as u64)
            .serialize(self)
    }
}
