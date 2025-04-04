use radicle::{git, identity, issue};

use radicle_types::domain::repo::models::cobs;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CreateIssueCommentPayload {
    pub rid: identity::RepoId,
    pub new: cobs::thread::NewIssueComment,
    pub opts: cobs::CobOptions,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CreateIssuePayload {
    pub rid: identity::RepoId,
    pub new: cobs::issue::NewIssue,
    pub opts: cobs::CobOptions,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditIssuePayload {
    pub rid: identity::RepoId,
    pub cob_id: git::Oid,
    pub action: cobs::issue::Action,
    pub opts: cobs::CobOptions,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssuesBody {
    pub rid: identity::RepoId,
    pub status: Option<cobs::query::IssueStatus>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct IssueBody {
    pub rid: identity::RepoId,
    pub id: issue::IssueId,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActivityBody {
    pub rid: identity::RepoId,
    pub id: git::Oid,
}
