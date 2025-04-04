use radicle::identity;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CreateEmbedBodyByPath {
    pub rid: identity::RepoId,
    pub path: std::path::PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CreateEmbedBodyByBytes {
    pub rid: identity::RepoId,
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CreateEmbedBodyByClipboard {
    pub rid: identity::RepoId,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SaveEmbedToDisk {
    pub rid: identity::RepoId,
    pub oid: radicle::git::Oid,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetEmbedPayload {
    pub rid: identity::RepoId,
    pub name: Option<String>,
    pub oid: radicle::git::Oid,
}
