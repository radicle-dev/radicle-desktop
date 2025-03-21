use std::collections::BTreeSet;

use localtime::LocalTime;
use radicle::identity::doc::PayloadId;
use radicle::issue::cache::Issues;
use radicle::node::routing::Store;
use radicle::patch::cache::Patches;
use radicle::prelude::Doc;
use radicle::storage::ReadRepository;
use radicle_surf as surf;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::{git, identity, issue, patch, storage, Profile};

use crate::domain::repo::models;
use crate::error::{self, Error};

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Show {
    Delegate,
    All,
    Contributor,
    Seeded,
    Private,
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(tag = "status")]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub enum SyncStatus {
    /// We're in sync.
    #[serde(rename_all = "camelCase")]
    Synced {
        /// At what ref was the remote synced at.
        at: SyncedAt,
    },
    /// We're out of sync.
    #[serde(rename_all = "camelCase")]
    OutOfSync {
        /// Local head of our `rad/sigrefs`.
        local: SyncedAt,
        /// Remote head of our `rad/sigrefs`.
        remote: SyncedAt,
    },
}

impl From<radicle::node::SyncStatus> for SyncStatus {
    fn from(value: radicle::node::SyncStatus) -> Self {
        match value {
            radicle::node::SyncStatus::Synced { at } => SyncStatus::Synced { at: at.into() },
            radicle::node::SyncStatus::OutOfSync { local, remote } => SyncStatus::OutOfSync {
                local: local.into(),
                remote: remote.into(),
            },
        }
    }
}

/// Holds an oid and timestamp.
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct SyncedAt {
    #[ts(as = "String")]
    pub oid: radicle::git::Oid,
    #[serde(with = "radicle::serde_ext::localtime::time")]
    #[ts(type = "number")]
    pub timestamp: LocalTime,
}

impl From<radicle::node::SyncedAt> for SyncedAt {
    fn from(value: radicle::node::SyncedAt) -> Self {
        Self {
            oid: value.oid,
            timestamp: value.timestamp,
        }
    }
}

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
    pub delegates: Vec<models::cobs::Author>,
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

pub fn repo_info(
    profile: &Profile,
    repo: &storage::git::Repository,
    doc: &Doc,
) -> Result<RepoInfo, Error> {
    let aliases = profile.aliases();
    let delegates = doc
        .delegates()
        .iter()
        .map(|did| super::cobs::Author::new(did, &aliases))
        .collect::<Vec<_>>();
    let db = profile.database()?;
    let seeding = db.count(&repo.id).unwrap_or_default();
    let (_, head) = repo.head()?;
    let commit = repo.commit(head)?;
    let project = doc
        .payload()
        .get(&PayloadId::project())
        .and_then(|payload| {
            let patches = profile.patches(repo).ok()?;
            let patches = patches.counts().ok()?;
            let issues = profile.issues(repo).ok()?;
            let issues = issues.counts().ok()?;

            let data: ProjectPayloadData = (*payload).clone().try_into().ok()?;
            let meta = ProjectPayloadMeta {
                issues,
                patches,
                head,
            };

            Some(ProjectPayload::new(data, meta))
        });

    Ok::<_, Error>(RepoInfo {
        payloads: SupportedPayloads { project },
        delegates,
        threshold: doc.threshold(),
        visibility: doc.visibility().clone().into(),
        rid: repo.id,
        seeding,
        last_commit_timestamp: commit.time().seconds() * 1000,
    })
}
