use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::cob;
use radicle::crypto;
use radicle::identity;
use radicle::issue;
use radicle::node::{Alias, AliasStore};
use radicle::patch;
use radicle::storage::git;

use crate::types::thread;

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    #[ts(as = "String")]
    did: identity::Did,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<String>")]
    #[ts(optional)]
    alias: Option<Alias>,
}

impl Author {
    pub fn new(did: identity::Did, aliases: &impl AliasStore) -> Self {
        Self {
            did,
            alias: aliases.alias(&did),
        }
    }
}

#[derive(TS, Serialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    #[ts(as = "String")]
    id: String,
    author: Author,
    title: String,
    #[ts(type = "{ status: 'closed', reason: 'other' | 'solved' } | { status: 'open' } ")]
    state: issue::State,
    assignees: Vec<Author>,
    discussion: Vec<thread::Comment>,
    #[ts(as = "Vec<String>")]
    labels: Vec<cob::Label>,
    #[ts(type = "number")]
    timestamp: cob::Timestamp,
}

impl Issue {
    pub fn new(id: &issue::IssueId, issue: &issue::Issue, aliases: &impl AliasStore) -> Self {
        Self {
            id: id.to_string(),
            author: Author::new(*issue.author().id(), aliases),
            title: issue.title().to_string(),
            state: *issue.state(),
            assignees: issue
                .assignees()
                .map(|did| Author::new(*did, aliases))
                .collect::<Vec<_>>(),
            discussion: issue
                .comments()
                .map(|(id, c)| thread::Comment::<Never>::new(*id, c.clone(), aliases))
                .collect::<Vec<_>>(),
            labels: issue.labels().cloned().collect::<Vec<_>>(),
            timestamp: issue.timestamp(),
        }
    }
}

#[derive(TS, Serialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    #[ts(as = "String")]
    id: String,
    author: Author,
    title: String,
    #[ts(as = "String")]
    base: git::Oid,
    #[ts(as = "String")]
    head: git::Oid,
    #[ts(type = r#"{
  status: 'draft'
} | {
  status: 'open',
  conflicts: [string, string][]
} | {
  status: 'archived'
} | {
  status: 'merged', revision: string, commit: string
} "#)]
    state: patch::State,
    assignees: Vec<Author>,
    #[ts(as = "Vec<String>")]
    labels: Vec<cob::Label>,
    #[ts(type = "number")]
    timestamp: cob::Timestamp,
    revision_count: usize,
}

impl Patch {
    pub fn new(id: patch::PatchId, patch: patch::Patch, aliases: &impl AliasStore) -> Self {
        Self {
            id: id.to_string(),
            author: Author::new(*patch.author().id(), aliases),
            title: patch.title().to_string(),
            state: patch.state().clone(),
            base: *patch.base(),
            head: *patch.head(),
            assignees: patch
                .assignees()
                .map(|did| Author::new(did, aliases))
                .collect::<Vec<_>>(),
            labels: patch.labels().cloned().collect::<Vec<_>>(),
            timestamp: patch.timestamp(),
            revision_count: patch.revisions().count(),
        }
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp.as_millis()
    }
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReviewEdit {
    #[ts(as = "String")]
    pub review_id: cob::patch::ReviewId,
    #[ts(as = "Option<String>")]
    pub verdict: Option<cob::patch::Verdict>,
    pub summary: Option<String>,
    #[ts(as = "Vec<String>")]
    pub labels: Vec<cob::Label>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Revision {
    #[ts(as = "String")]
    id: patch::RevisionId,
    author: Author,
    description: Vec<Edit>,
    #[ts(as = "String")]
    base: git::Oid,
    #[ts(as = "String")]
    head: git::Oid,
    reviews: Vec<Review>,
    #[ts(type = "number")]
    timestamp: cob::common::Timestamp,
    discussion: Vec<thread::Comment<thread::CodeLocation>>,
    reactions: Vec<Reaction>,
}

impl Revision {
    pub fn new(value: cob::patch::Revision, aliases: &impl AliasStore) -> Self {
        Self {
            id: value.id(),
            author: Author::new(*value.author().id(), aliases),
            description: value
                .edits()
                .map(|e| Edit::new(e, aliases))
                .collect::<Vec<_>>(),
            base: *value.base(),
            head: value.head(),
            reviews: value
                .reviews()
                .map(|(id, r)| Review::new(*id, r.clone(), aliases))
                .collect::<Vec<_>>(),
            timestamp: value.timestamp(),
            discussion: value
                .discussion()
                .comments()
                .map(|(id, c)| {
                    thread::Comment::<thread::CodeLocation>::new(*id, c.clone(), aliases)
                })
                .collect::<Vec<_>>(),
            reactions: value
                .reactions()
                .iter()
                .flat_map(|(location, reactions)| {
                    let reaction_by_author = reactions.iter().fold(
                        BTreeMap::new(),
                        |mut acc: BTreeMap<&cob::Reaction, Vec<_>>, (author, emoji)| {
                            acc.entry(emoji).or_default().push(author);
                            acc
                        },
                    );
                    reaction_by_author
                        .into_iter()
                        .map(|(emoji, authors)| {
                            Reaction::new(
                                *emoji,
                                authors,
                                location
                                    .as_ref()
                                    .map(|l| thread::CodeLocation::new(l.clone())),
                                aliases,
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Reaction {
    #[ts(as = "String")]
    emoji: cob::Reaction,
    authors: Vec<Author>,
    #[ts(optional)]
    location: Option<thread::CodeLocation>,
}

impl Reaction {
    pub fn new(
        emoji: cob::Reaction,
        authors: Vec<&crypto::PublicKey>,
        location: Option<thread::CodeLocation>,
        aliases: &impl AliasStore,
    ) -> Self {
        Self {
            emoji,
            authors: authors
                .into_iter()
                .map(|a| Author::new(a.into(), aliases))
                .collect::<Vec<_>>(),
            location,
        }
    }
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    #[ts(as = "String")]
    id: identity::PublicKey,
    author: Author,
    #[ts(type = "'accept' | 'reject'")]
    #[ts(optional)]
    verdict: Option<cob::patch::Verdict>,
    summary: Option<String>,
    comments: Vec<thread::Comment<thread::CodeLocation>>,
    #[ts(type = "number")]
    timestamp: cob::common::Timestamp,
}

impl Review {
    pub fn new(
        id: identity::PublicKey,
        review: cob::patch::Review,
        aliases: &impl AliasStore,
    ) -> Self {
        Self {
            id,
            author: Author::new(review.author().id, aliases),
            verdict: review.verdict(),
            summary: review.summary().map(|s| s.to_string()),
            comments: review
                .comments()
                .map(|(id, c)| {
                    thread::Comment::<thread::CodeLocation>::new(*id, c.clone(), aliases)
                })
                .collect::<Vec<_>>(),
            timestamp: review.timestamp(),
        }
    }
}

#[derive(TS, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    pub author: Author,
    #[ts(type = "number")]
    pub timestamp: cob::common::Timestamp,
    pub body: String,
    #[ts(type = "{ name: string, content: string }")]
    pub embeds: Vec<cob::change::store::Embed<cob::common::Uri>>,
}

impl Edit {
    pub fn new(edit: &cob::thread::Edit, aliases: &impl AliasStore) -> Self {
        Self {
            author: Author::new(edit.author.into(), aliases),
            timestamp: edit.timestamp,
            body: edit.body.clone(),
            embeds: edit.embeds.clone(),
        }
    }
}

/// The `Infallible` type does not have a `Serialize`/`Deserialize`
/// implementation. The `Never` type imitates `Infallible` and
/// provides the derived implementations.
#[derive(TS, Serialize)]
pub enum Never {}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct NewIssue {
    pub title: String,
    pub description: String,
    #[ts(as = "Vec<String>")]
    pub labels: Vec<cob::Label>,
    #[ts(as = "Vec<String>")]
    pub assignees: Vec<identity::Did>,
    #[ts(type = "{ name: string, content: string }[]")]
    pub embeds: Vec<cob::Embed>,
}

#[derive(TS, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CobOptions {
    #[ts(as = "Option<bool>")]
    #[ts(optional)]
    announce: Option<bool>,
}

impl CobOptions {
    pub fn announce(&self) -> bool {
        self.announce.unwrap_or(true)
    }
}

#[derive(TS, Serialize)]
#[ts(export)]
pub struct Stats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl Stats {
    pub fn new(stats: &radicle_surf::diff::Stats) -> Self {
        Self {
            files_changed: stats.files_changed,
            insertions: stats.insertions,
            deletions: stats.deletions,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export)]
pub enum IssueAction {
    /// Assign issue to an actor.
    #[serde(rename = "assign")]
    Assign {
        #[ts(as = "Vec<String>")]
        assignees: BTreeSet<identity::Did>,
    },

    /// Edit issue title.
    #[serde(rename = "edit")]
    Edit { title: String },

    /// Transition to a different state.
    #[serde(rename = "lifecycle")]
    Lifecycle {
        #[ts(type = "{ status: 'closed', reason: 'other' | 'solved' } | { status: 'open' } ")]
        state: issue::State,
    },

    /// Modify issue labels.
    #[serde(rename = "label")]
    Label {
        #[ts(as = "Vec<String>")]
        labels: BTreeSet<cob::Label>,
    },

    /// Comment on a thread.
    #[serde(rename_all = "camelCase")]
    #[serde(rename = "comment")]
    Comment {
        /// Comment body.
        body: String,
        /// Comment this is a reply to.
        /// Should be [`None` | `null`] if it's the top-level comment.
        /// Should be the root [`CommentId`] if it's a top-level comment.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(as = "Option<String>")]
        reply_to: Option<cob::thread::CommentId>,
        /// Embeded content.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(as = "Vec<String>")]
        embeds: Vec<cob::Embed<cob::Uri>>,
    },

    /// Edit a comment.
    #[serde(rename = "comment.edit")]
    CommentEdit {
        /// Comment being edited.
        #[ts(as = "String")]
        id: cob::thread::CommentId,
        /// New value for the comment body.
        body: String,
        /// New value for the embeds list.
        #[ts(as = "Vec<String>")]
        embeds: Vec<cob::Embed<cob::Uri>>,
    },

    /// Redact a change. Not all changes can be redacted.
    #[serde(rename = "comment.redact")]
    CommentRedact {
        #[ts(as = "String")]
        id: cob::thread::CommentId,
    },

    /// React to a comment.
    #[serde(rename = "comment.react")]
    CommentReact {
        #[ts(as = "String")]
        id: cob::thread::CommentId,
        #[ts(as = "String")]
        reaction: cob::Reaction,
        active: bool,
    },
}
