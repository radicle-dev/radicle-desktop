use std::collections::BTreeSet;

use radicle_surf as surf;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::{git, identity, issue, patch};

use crate::cobs::Author;
use crate::error;

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct RepoCount {
    pub total: usize,
    pub contributor: usize,
    pub delegate: usize,
    pub private: usize,
    pub seeding: usize,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct RepoInfo {
    pub payloads: SupportedPayloads,
    pub delegates: Vec<Author>,
    pub threshold: usize,
    pub visibility: Visibility,
    #[ts(as = "String")]
    pub rid: identity::RepoId,
    pub seeding: usize,
    #[ts(type = "number")]
    pub last_commit_timestamp: i64,
}

#[derive(Default, Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub enum Visibility {
    /// Anyone and everyone.
    #[default]
    Public,
    /// Delegates plus the allowed DIDs.
    Private {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        #[ts(as = "Option<BTreeSet<String>>", optional)]
        allow: BTreeSet<identity::Did>,
    },
}

impl From<identity::Visibility> for Visibility {
    fn from(value: identity::Visibility) -> Self {
        match value {
            identity::Visibility::Private { allow } => Self::Private { allow },
            identity::Visibility::Public => Self::Public,
        }
    }
}

impl From<Visibility> for identity::Visibility {
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Private { allow } => Self::Private { allow },
            Visibility::Public => Self::Public,
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct SupportedPayloads {
    #[serde(rename = "xyz.radicle.project")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub project: Option<ProjectPayload>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct ProjectPayload {
    data: ProjectPayloadData,
    meta: ProjectPayloadMeta,
}

impl ProjectPayload {
    pub fn new(data: ProjectPayloadData, meta: ProjectPayloadMeta) -> Self {
        Self { data, meta }
    }
}

impl TryFrom<identity::doc::Payload> for ProjectPayloadData {
    type Error = error::Error;

    fn try_from(value: identity::doc::Payload) -> Result<Self, Self::Error> {
        serde_json::from_value::<Self>((*value).clone()).map_err(Into::into)
    }
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct ProjectPayloadData {
    pub default_branch: String,
    pub description: String,
    pub name: String,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct ProjectPayloadMeta {
    #[ts(as = "String")]
    pub head: git::Oid,
    #[ts(type = "{ open: number, closed: number }")]
    pub issues: issue::IssueCounts,
    #[ts(type = "{ open: number, draft: number, archived: number, merged: number }")]
    pub patches: patch::PatchCounts,
}

#[derive(Clone, Serialize, TS, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct Commit {
    #[ts(as = "String")]
    pub id: git::Oid,
    #[ts(type = "{ name: string; email: string; time: number; }")]
    pub author: surf::Author,
    #[ts(type = "{ name: string; email: string; time: number; }")]
    pub committer: surf::Author,
    pub message: String,
    pub summary: String,
    #[ts(as = "Vec<String>")]
    pub parents: Vec<git::Oid>,
}

impl From<surf::Commit> for Commit {
    fn from(value: surf::Commit) -> Self {
        Self {
            id: value.id,
            author: value.author,
            committer: value.committer,
            message: value.message,
            summary: value.summary,
            parents: value.parents,
        }
    }
}
