use crate::{
    constants::UNIX_SOCKET_NAME,
    message::{Request, Response},
};
use lazy_static::lazy_static;
use std::{
    io::{Read, Write},
    net::Shutdown,
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
};
use thiserror::Error;

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

    fn connect() -> Result<UnixStream, anyhow::Error> {
        let path = SOCKET_PATH.to_owned();
        Ok(UnixStream::connect(path)?)
    }

    pub fn send_with_response(self) -> Result<Response, SocketError> {
        let mut stream = Self::connect()?;
        let buffer = bincode::serialize(&self.request.unwrap_or_default())?;
        stream.write_all(&buffer)?;

        stream.shutdown(Shutdown::Write)?;

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;

        let response = bincode::deserialize(&buffer)?;

        Ok(response)
    }

    pub fn send(self) -> Result<(), SocketError> {
        let mut stream = Self::connect()?;
        let buffer = bincode::serialize(&self.request.unwrap_or_default())?;
        stream.write_all(&buffer)?;

        Ok(())
    }
}

pub fn bind() -> Result<UnixListener, anyhow::Error> {
    let path = SOCKET_PATH.to_owned();

    if Path::new(&path).exists() {
        std::fs::remove_file(&path)?;
        tracing::info!("socket already existed, removed");
    }

    Ok(UnixListener::bind(path)?)
}

pub fn disconnect() -> Result<(), anyhow::Error> {
    let path = SOCKET_PATH.to_owned();
    Ok(std::fs::remove_file(path)?)
}
