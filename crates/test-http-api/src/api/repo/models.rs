use radicle::{git, identity};

use radicle_types::domain::repo::models::{diff, repo};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DiffPayload {
    pub rid: identity::RepoId,
    pub base: git::Oid,
    pub head: git::Oid,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct GetDiffPayload {
    pub rid: identity::RepoId,
    pub options: diff::DiffOptions,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct RepoRootOptions {
    pub show: repo::Show,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct RepoPayload {
    pub rid: identity::RepoId,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CreateRepoPayload {
    pub name: String,
    pub description: String,
}
