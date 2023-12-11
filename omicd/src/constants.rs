pub const UNIX_SOCKET_NAME: &str = "omic-socket";

#[repr(u8)]
pub enum Message {
    Connect = 1,
    Disconnect = 0,
    Hello = 2,
}

pub const MAX_MESSAGE_SIZE: usize = 1024;
