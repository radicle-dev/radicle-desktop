#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct AuthPayload {
    pub passphrase: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Init {
    pub alias: String,
    pub passphrase: String,
}
