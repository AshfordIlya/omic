use std::{
    net::TcpStream,
    os::fd::{AsRawFd, RawFd},
};

use crate::constants::MAX_MESSAGE_SIZE;
use bincode::Options;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Request {
    Status,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Response {
    //Ok,
    Connection { port: i32 },
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

pub struct State {
    pub socket: uds::UnixSeqpacketListener,
    pub port: u16,
}

impl AsRawFd for State {
    fn as_raw_fd(&self) -> RawFd {
        self.socket.as_raw_fd()
    }
}
