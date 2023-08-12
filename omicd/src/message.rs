#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum Request {
    #[default]
    Noop,
    Connect {
        address: String,
        port: String,
    },
    Disconnect,
    Query,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Response {
    Ok,
    Error(String),
}
