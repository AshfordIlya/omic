#[derive(serde::Serialize, serde::Deserialize)]
pub enum Message {
    Connect { address: String, port: String },
    Disconnect,
    Error(String),
}
