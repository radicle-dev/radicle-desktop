use std::collections::{BTreeMap, BTreeSet};

use radicle_surf as surf;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::node::Alias;
use radicle::{git, identity, issue, node, patch};

use crate::cobs::Author;
use crate::error;

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct RepoSummary {
    #[ts(as = "String")]
    pub rid: identity::RepoId,
    pub name: String,
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
    pub delegates: Vec<Author>,
    pub threshold: usize,
    pub visibility: Visibility,
    #[ts(as = "String")]
    pub rid: identity::RepoId,
    pub seeding: usize,
    #[ts(type = "number")]
    pub last_commit_timestamp: i64,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct Readme {
    #[ts(as = "String")]
    pub id: surf::Oid,
    pub binary: bool,
    pub commit: Commit,
    pub mime_type: String,
    pub content: String,
    pub path: String,
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(as = "Option<Vec<Author>>", optional)]
        allow: Vec<Author>,
    },
}

impl From<Visibility> for identity::Visibility {
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Private { allow } => {
                let did_set = allow
                    .iter()
                    .map(|author| *author.did())
                    .collect::<BTreeSet<identity::Did>>();
                Self::Private { allow: did_set }
            }
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

    pub fn name(&self) -> &str {
        &self.data.name
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

/// Outcome of checking a commit's `gpgsig` header.
///
/// Absence of the header is not represented here: an unsigned commit carries no
/// [`CommitSignature`] at all.
#[derive(Clone, Copy, Serialize, TS, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub enum SignatureStatus {
    /// An `sshsig` over the `git` namespace that verifies against the Ed25519
    /// key embedded in the signature.
    Verified,
    /// A signature that is well-formed but does not verify over the commit
    /// payload.
    Invalid,
    /// A signature we make no claim about: PGP, a non-Ed25519 key, a namespace
    /// other than `git`, or a container we cannot parse.
    Unsupported,
}

#[derive(Clone, Serialize, TS, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct CommitSignature {
    pub status: SignatureStatus,
    /// The signing key, as a Radicle identity. Git signs with whatever
    /// `user.signingKey` points at; when an author points that at their Radicle
    /// key, the Ed25519 key carried in the signature is also their node ID and
    /// so resolves to a DID. Absent when the key is not Ed25519.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub signer: Option<Author>,
    /// Whether the signer is a delegate of this repository.
    pub delegate: bool,
    /// Whether the signer was a delegate under some earlier identity revision,
    /// but is not one now.
    ///
    /// This is deliberately an existence claim over the identity history rather
    /// than a claim about the moment this commit was signed. Nothing in a commit
    /// object points at an identity revision, so the only way to date a commit
    /// against the delegate set is its committer timestamp — which the signer
    /// chooses freely, and can therefore backdate into a delegacy window they no
    /// longer hold.
    pub former_delegate: bool,
    /// Whether the signer has a remote in this repository.
    pub remote: bool,
    /// Whether the signing key corresponds to a node we have any evidence for:
    /// it is a delegate, was one, has a remote here, or is a node the local
    /// node knows by alias.
    ///
    /// Signing a git commit with a Radicle key is a convention an author opts
    /// into, not something the protocol does, so any Ed25519 key encodes to a
    /// syntactically valid DID whether or not a node behind it exists. Without
    /// one of the above, that DID is not evidence of a Radicle identity and
    /// must not be presented as one.
    pub known: bool,
    /// OpenSSH fingerprint of the signing key, in the form
    /// `git show --show-signature` prints. Present whenever the key is Ed25519.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fingerprint: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub signature: Option<CommitSignature>,
}

impl Commit {
    #[must_use]
    pub fn with_signature(mut self, signature: Option<CommitSignature>) -> Self {
        self.signature = signature;
        self
    }
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct Tagger {
    pub name: String,
    pub email: String,
    /// Tagger time. Seconds since epoch.
    #[ts(type = "number")]
    pub timestamp: i64,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct Tag {
    #[ts(as = "String")]
    pub oid: git::Oid,
    /// Tagger time for annotated tags, otherwise the commit time of the
    /// tagged commit. Seconds since epoch.
    #[ts(type = "number")]
    pub timestamp: i64,
    /// Tagger of an annotated tag. Absent for lightweight tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tagger: Option<Tagger>,
    /// Message of an annotated tag. Absent for lightweight tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub message: Option<String>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct Remote {
    #[ts(as = "String")]
    pub id: node::NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<String>", optional)]
    pub alias: Option<Alias>,
    pub delegate: bool,
    #[ts(as = "BTreeMap<String, String>")]
    pub branches: BTreeMap<String, git::Oid>,
    pub tags: BTreeMap<String, Tag>,
}

#[derive(Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct Canonical {
    #[ts(as = "BTreeMap<String, String>")]
    pub branches: BTreeMap<String, git::Oid>,
    pub tags: BTreeMap<String, Tag>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "repo/")]
pub struct RepoRefs {
    pub canonical: Canonical,
    pub remotes: Vec<Remote>,
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
            signature: None,
        }
    }
}
