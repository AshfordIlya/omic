#[derive(serde::Serialize, serde::Deserialize)]
pub enum Request {
    Connect { address: String, port: String },
    Disconnect,
    Query,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Response {
    Ok,
    Error(String),
}
