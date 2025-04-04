use radicle::{git, identity, patch};

use radicle_types::domain::repo::models::cobs;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActivityBody {
    pub rid: identity::RepoId,
    pub id: git::Oid,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReviewByPatchPayload {
    pub rid: identity::RepoId,
    pub id: patch::PatchId,
    pub revision_id: patch::RevisionId,
    pub review_id: patch::ReviewId,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PatchesBody {
    pub rid: radicle::identity::RepoId,
    pub skip: Option<usize>,
    pub take: Option<usize>,
    pub status: Option<cobs::query::PatchStatus>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct PatchBody {
    pub rid: radicle::identity::RepoId,
    pub id: radicle::git::Oid,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditPatchPayload {
    pub rid: identity::RepoId,
    pub cob_id: radicle::git::Oid,
    pub action: cobs::patch::Action,
    pub opts: cobs::CobOptions,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct PatchRevisionsPayload {
    pub rid: identity::RepoId,
    pub id: patch::PatchId,
}
