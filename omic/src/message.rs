#[derive(serde::Serialize, serde::Deserialize)]
pub enum Request {
    Connect { address: String, port: String },
    Disconnect,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Response {
    Ok,
    Error(String),
}
