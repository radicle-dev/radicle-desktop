use std::collections::BTreeMap;

use radicle::cob;
use radicle::git;
use radicle::node::AliasStore;
use radicle::storage::git::Repository;
use serde::Serialize;
use ts_rs::TS;

use radicle_artifact::display::{CommitTitle, TagName};

use crate::cobs;

/// A place an artifact can be fetched from, contributed by a single node.
#[derive(Clone, Serialize, TS, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/release/")]
pub struct Location {
    pub user: cobs::Author,
    pub url: String,
}

/// A node flagging an artifact, with the reason it gave.
#[derive(Clone, Serialize, TS, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/release/")]
pub struct Redaction {
    pub user: cobs::Author,
    pub reason: String,
}

/// A single file published as part of a release, addressed by its content id.
#[derive(Clone, Serialize, TS, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/release/")]
pub struct Artifact {
    pub cid: String,
    pub name: String,
    pub author: cobs::Author,
    /// Flattened across contributors: one entry per node/url pair.
    pub locations: Vec<Location>,
    pub attestations: Vec<cobs::Author>,
    pub redactions: Vec<Redaction>,
    #[ts(type = "Record<string, unknown>")]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

impl Artifact {
    pub fn new(
        cid: &radicle_artifact::Cid,
        artifact: &radicle_artifact::Artifact,
        aliases: &impl AliasStore,
    ) -> Self {
        Self {
            cid: cid.to_string(),
            name: artifact.name().to_string(),
            author: cobs::Author::new(artifact.author(), aliases),
            locations: artifact
                .locations()
                .iter()
                .flat_map(|(did, urls)| {
                    urls.iter().map(move |url| Location {
                        user: cobs::Author::new(did, aliases),
                        url: url.to_string(),
                    })
                })
                .collect(),
            attestations: artifact
                .attestations()
                .iter()
                .map(|did| cobs::Author::new(did, aliases))
                .collect(),
            redactions: artifact
                .redactions()
                .iter()
                .map(|(did, reason)| Redaction {
                    user: cobs::Author::new(did, aliases),
                    reason: reason.clone(),
                })
                .collect(),
            metadata: artifact.metadata().clone(),
        }
    }
}

/// A release, keyed by a commit and optionally an annotated tag. The COB has
/// no title of its own: `title` and `tagName` are resolved from the tag or
/// commit message, and are absent when they cannot be read.
#[derive(Clone, Serialize, TS, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/release/")]
pub struct Release {
    #[ts(as = "String")]
    pub id: git::Oid,
    #[ts(as = "String")]
    pub oid: git::Oid,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<String>", optional)]
    pub tag: Option<git::Oid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
    #[ts(type = "number")]
    pub created_at: cob::Timestamp,
    pub creator: cobs::Author,
    pub artifacts: Vec<Artifact>,
}

impl Release {
    /// Build a release from its COB, with `artifacts` already narrowed to the
    /// ones visible under the caller's filter.
    pub fn new(
        id: radicle_artifact::ReleaseId,
        release: &radicle_artifact::Release,
        repo: &Repository,
        aliases: &impl AliasStore,
        artifacts: Vec<Artifact>,
    ) -> Self {
        let title = release
            .tag()
            .and_then(|tag| repo.title(tag))
            .or_else(|| repo.title(release.oid()));
        let tag_name = release.tag().and_then(|tag| repo.tag_name(tag));

        Self {
            id: id.oid(),
            oid: *release.oid(),
            tag: release.tag().copied(),
            tag_name,
            title,
            created_at: release.timestamp(),
            creator: cobs::Author::new(release.creator(), aliases),
            artifacts,
        }
    }
}
