use std::path::PathBuf;

use radicle::identity;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CreateEmbedBody {
    pub rid: identity::RepoId,
    pub path: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetEmbedPayload {
    pub rid: identity::RepoId,
    pub name: Option<String>,
    pub oid: radicle::git::Oid,
}
