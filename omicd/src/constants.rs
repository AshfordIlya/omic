pub const UNIX_SOCKET_NAME: &str = "omic-socket";

#[repr(u8)]
pub enum UdpSocketMessage {
    Connect = 1,
    Disconnect = 0,
}

pub const MAX_MESSAGE_SIZE: usize = 1024;
