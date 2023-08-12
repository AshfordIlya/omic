use crate::{
    constants::{MAX_MESSAGE_SIZE, UNIX_SOCKET_NAME},
    message::{Request, Response},
};
use lazy_static::lazy_static;
use std::path::Path;
use thiserror::Error;
use uds::{UnixSeqpacketConn, UnixSeqpacketListener};

lazy_static! {
    static ref SOCKET_PATH: String = match std::env::var("XDG_RUNTIME_DIR") {
        Ok(runtime_dir) => format!("{runtime_dir}/{UNIX_SOCKET_NAME}"),
        Err(_) => format!(
            "{}/.{UNIX_SOCKET_NAME}",
            std::env::var("HOME").unwrap_or("/tmp".to_owned())
        ),
    };
}

pub struct Socket;
impl Socket {
    pub fn create_request() -> SocketRequestBuilder {
        SocketRequestBuilder::default()
    }
}

#[derive(Error, Debug)]
pub enum SocketError {
    #[error("Unknown error ocurred")]
    Unknown(#[from] anyhow::Error),

    #[error("Stream error ocurred")]
    StreamError(#[from] std::io::Error),

    #[error("Serializer error ocurred")]
    SerializerError(#[from] bincode::Error),
}

#[derive(Default)]
pub struct SocketRequestBuilder {
    request: Option<Request>,
}

impl SocketRequestBuilder {
    pub fn request(mut self, req: Request) -> Self {
        self.request = Some(req);
        self
    }

    fn connect() -> Result<UnixSeqpacketConn, anyhow::Error> {
        let path = SOCKET_PATH.to_owned();
        Ok(UnixSeqpacketConn::connect(path)?)
    }

    pub fn send_with_response(self) -> Result<Response, SocketError> {
        let stream = Self::connect()?;
        stream.send(&self.request.unwrap_or_default().to_bytes()?)?;

        let mut buffer = [0; MAX_MESSAGE_SIZE];
        stream.recv(&mut buffer)?;

        let response = bincode::deserialize(&buffer)?;

        Ok(response)
    }

    pub fn send(self) -> Result<(), SocketError> {
        let stream = Self::connect()?;
        stream.send(&self.request.unwrap_or_default().to_bytes()?)?;

        Ok(())
    }
}

pub fn bind() -> Result<UnixSeqpacketListener, anyhow::Error> {
    let path = SOCKET_PATH.to_owned();

    if Path::new(&path).exists() {
        tracing::info!("socket already exists, removing");
        std::fs::remove_file(&path)?;
    }

    Ok(UnixSeqpacketListener::bind(path)?)
}

pub fn disconnect() -> Result<(), anyhow::Error> {
    let path = SOCKET_PATH.to_owned();
    Ok(std::fs::remove_file(path)?)
}
