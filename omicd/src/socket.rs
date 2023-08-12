use crate::constants::UNIX_SOCKET_NAME;
use lazy_static::lazy_static;
use std::{
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
};

lazy_static! {
    static ref SOCKET_PATH: String = match std::env::var("XDG_RUNTIME_DIR") {
        Ok(runtime_dir) => format!("{runtime_dir}/{UNIX_SOCKET_NAME}"),
        Err(_) => format!(
            "{}/.{UNIX_SOCKET_NAME}",
            std::env::var("HOME").unwrap_or("/tmp".to_owned())
        ),
    };
}

pub fn bind() -> Result<UnixListener, anyhow::Error> {
    let path = SOCKET_PATH.to_owned();

    if Path::new(&path).exists() {
        std::fs::remove_file(&path)?;
        tracing::info!("socket already existed, removed");
    }

    Ok(UnixListener::bind(path)?)
}

pub fn connect() -> Result<UnixStream, anyhow::Error> {
    let path = SOCKET_PATH.to_owned();
    Ok(UnixStream::connect(path)?)
}

pub fn disconnect() -> Result<(), anyhow::Error> {
    let path = SOCKET_PATH.to_owned();
    Ok(std::fs::remove_file(path)?)
}
