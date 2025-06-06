use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ops::Index;

use radicle::node::AliasStore;
use radicle::patch::Status;
use radicle::profile::Aliases;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::cob;
use radicle::git;
use radicle::patch;

use crate::cobs;
use crate::cobs::Author;
use crate::cobs::FromRadicleAction;

#[derive(Debug, TS, Serialize)]
#[ts(export)]
#[ts(export_to = "cob/patch/")]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    id: String,
    author: cobs::Author,
    title: String,
    #[ts(as = "String")]
    base: git::Oid,
    #[ts(as = "String")]
    head: git::Oid,
    state: State,
    assignees: Vec<cobs::Author>,
    #[ts(as = "Vec<String>")]
    labels: Vec<cob::Label>,
    #[ts(type = "number")]
    timestamp: cob::Timestamp,
    revision_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ListPatchesError {
    #[error(transparent)]
    Sqlite(#[from] sqlite::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    // to be extended as new error scenarios are introduced
}

impl Patch {
    pub fn new(id: patch::PatchId, patch: &patch::Patch, aliases: &impl AliasStore) -> Self {
        Self {
            id: id.to_string(),
            author: cobs::Author::new(patch.author().id(), aliases),
            title: patch.title().to_string(),
            state: patch.state().clone().into(),
            base: *patch.base(),
            head: *patch.head(),
            assignees: patch
                .assignees()
                .map(|did| cobs::Author::new(&did, aliases))
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

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", tag = "status")]
#[ts(export)]
#[ts(export_to = "cob/patch/")]
pub enum State {
    Draft,
    Open {
        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default)]
        #[ts(as = "Option<Vec<(String, String)>>", optional)]
        conflicts: Vec<(patch::RevisionId, git::Oid)>,
    },
    Archived,
    Merged {
        #[ts(as = "String")]
        revision: patch::RevisionId,
        #[ts(as = "String")]
        commit: git::Oid,
    },
}

impl From<State> for patch::State {
    fn from(value: State) -> Self {
        match value {
            State::Archived => Self::Archived,
            State::Draft => Self::Draft,
            State::Merged { revision, commit } => Self::Merged { revision, commit },
            State::Open { conflicts } => Self::Open { conflicts },
        }
    }
}

impl From<patch::State> for State {
    fn from(value: patch::State) -> Self {
        match value {
            patch::State::Archived => Self::Archived,
            patch::State::Draft => Self::Draft,
            patch::State::Merged { revision, commit } => Self::Merged { revision, commit },
            patch::State::Open { conflicts } => Self::Open { conflicts },
        }
    }
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/patch/")]
pub struct ReviewEdit {
    #[ts(as = "String")]
    pub review_id: cob::patch::ReviewId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub verdict: Option<Verdict>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub summary: Option<String>,
    #[ts(as = "Option<Vec<String>>", optional)]
    pub labels: Vec<cob::Label>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/patch/")]
pub struct Revision {
    #[ts(as = "String")]
    id: patch::RevisionId,
    author: cobs::Author,
    description: Vec<Edit>,
    #[ts(as = "String")]
    base: git::Oid,
    #[ts(as = "String")]
    head: git::Oid,
    #[ts(as = "Option<_>", optional)]
    reviews: Vec<Review>,
    #[ts(type = "number")]
    timestamp: cob::common::Timestamp,
    #[ts(as = "Option<_>", optional)]
    discussion: Vec<cobs::thread::Comment<cobs::thread::CodeLocation>>,
    #[ts(as = "Option<_>", optional)]
    reactions: Vec<cobs::thread::Reaction>,
}

impl Revision {
    pub fn new(value: cob::patch::Revision, aliases: &impl AliasStore) -> Self {
        Self {
            id: value.id(),
            author: cobs::Author::new(value.author().id(), aliases),
            description: value
                .edits()
                .map(|e| Edit::new(e, aliases))
                .collect::<Vec<_>>(),
            base: *value.base(),
            head: value.head(),
            reviews: value
                .reviews()
                .map(|(_, r)| Review::new(r.clone(), aliases))
                .collect::<Vec<_>>(),
            timestamp: value.timestamp(),
            discussion: value
                .discussion()
                .comments()
                .map(|(id, c)| {
                    cobs::thread::Comment::<cobs::thread::CodeLocation>::new(
                        *id,
                        c.clone(),
                        aliases,
                    )
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
                            cobs::thread::Reaction::new(
                                *emoji,
                                authors.into_iter().map(Into::into).collect::<Vec<_>>(),
                                location
                                    .as_ref()
                                    .map(|l| cobs::thread::CodeLocation::new(l.clone())),
                                aliases,
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(TS, Serialize)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/patch/")]
pub struct Edit {
    pub author: cobs::Author,
    #[ts(type = "number")]
    pub timestamp: cob::common::Timestamp,
    pub body: String,
    #[ts(as = "Option<_>", optional)]
    pub embeds: Vec<cobs::thread::Embed>,
}

impl Edit {
    pub fn new(edit: &cob::thread::Edit, aliases: &impl AliasStore) -> Self {
        Self {
            author: cobs::Author::new(&edit.author.into(), aliases),
            timestamp: edit.timestamp,
            body: edit.body.clone(),
            embeds: edit
                .embeds
                .iter()
                .cloned()
                .map(|e| e.into())
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/patch/")]
pub struct Review {
    #[ts(as = "String")]
    id: cob::patch::ReviewId,
    author: cobs::Author,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    verdict: Option<Verdict>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    summary: Option<String>,
    comments: Vec<cobs::thread::Comment<cobs::thread::CodeLocation>>,
    #[ts(type = "number")]
    timestamp: cob::common::Timestamp,
    #[ts(as = "Vec<String>")]
    labels: Vec<cob::Label>,
}

impl Review {
    pub fn new(review: cob::patch::Review, aliases: &impl AliasStore) -> Self {
        Self {
            id: review.id(),
            author: cobs::Author::new(&review.author().id, aliases),
            verdict: review.verdict().map(|v| v.into()),
            summary: review.summary().map(|s| s.to_string()),
            labels: review.labels().cloned().collect::<Vec<_>>(),
            comments: review
                .comments()
                .map(|(id, c)| {
                    cobs::thread::Comment::<cobs::thread::CodeLocation>::new(
                        *id,
                        c.clone(),
                        aliases,
                    )
                })
                .collect::<Vec<_>>(),
            timestamp: review.timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/patch/")]
pub enum Verdict {
    Accept,
    Reject,
}

impl From<cob::patch::Verdict> for Verdict {
    fn from(value: cob::patch::Verdict) -> Self {
        match value {
            cob::patch::Verdict::Accept => Self::Accept,
            cob::patch::Verdict::Reject => Self::Reject,
        }
    }
}

impl From<Verdict> for cob::patch::Verdict {
    fn from(value: Verdict) -> Self {
        match value {
            Verdict::Accept => Self::Accept,
            Verdict::Reject => Self::Reject,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/patch/")]
pub enum Action {
    #[serde(rename = "edit")]
    Edit {
        title: String,
        #[ts(as = "String")]
        target: patch::MergeTarget,
    },
    #[serde(rename = "label")]
    Label {
        #[ts(as = "Vec<String>")]
        labels: BTreeSet<cob::Label>,
    },
    #[serde(rename = "lifecycle")]
    Lifecycle {
        #[ts(type = "{ status: 'draft' | 'open' | 'archived' }")]
        state: patch::Lifecycle,
    },
    #[serde(rename = "assign")]
    Assign { assignees: BTreeSet<cobs::Author> },
    #[serde(rename = "merge")]
    Merge {
        #[ts(as = "String")]
        revision: patch::RevisionId,
        #[ts(as = "String")]
        commit: git::Oid,
    },

    #[serde(rename = "review")]
    Review {
        #[ts(as = "String")]
        revision: patch::RevisionId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        summary: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        verdict: Option<Verdict>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(as = "Option<Vec<String>>", optional)]
        labels: Vec<cob::Label>,
    },
    #[serde(rename = "review.edit")]
    ReviewEdit {
        #[ts(as = "String")]
        review: patch::ReviewId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        summary: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        verdict: Option<Verdict>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(as = "Option<Vec<String>>", optional)]
        labels: Vec<cob::Label>,
    },
    #[serde(rename = "review.redact")]
    ReviewRedact {
        #[ts(as = "String")]
        review: patch::ReviewId,
    },
    #[serde(rename = "review.comment")]
    #[serde(rename_all = "camelCase")]
    ReviewComment {
        #[ts(as = "String")]
        review: patch::ReviewId,
        body: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        location: Option<cobs::thread::CodeLocation>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(as = "Option<String>", optional)]
        reply_to: Option<cob::thread::CommentId>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(as = "Option<_>", optional)]
        embeds: Vec<cobs::thread::Embed>,
    },
    #[serde(rename = "review.comment.edit")]
    ReviewCommentEdit {
        #[ts(as = "String")]
        review: patch::ReviewId,
        #[ts(as = "String")]
        comment: cob::EntryId,
        body: String,
        #[ts(as = "Option<_>", optional)]
        embeds: Vec<cobs::thread::Embed>,
    },
    #[serde(rename = "review.comment.redact")]
    ReviewCommentRedact {
        #[ts(as = "String")]
        review: patch::ReviewId,
        #[ts(as = "String")]
        comment: cob::EntryId,
    },
    #[serde(rename = "review.comment.react")]
    ReviewCommentReact {
        #[ts(as = "String")]
        review: patch::ReviewId,
        #[ts(as = "String")]
        comment: cob::EntryId,
        #[ts(as = "String")]
        reaction: cob::Reaction,
        active: bool,
    },
    #[serde(rename = "review.comment.resolve")]
    ReviewCommentResolve {
        #[ts(as = "String")]
        review: patch::ReviewId,
        #[ts(as = "String")]
        comment: cob::EntryId,
    },
    #[serde(rename = "review.comment.unresolve")]
    ReviewCommentUnresolve {
        #[ts(as = "String")]
        review: patch::ReviewId,
        #[ts(as = "String")]
        comment: cob::EntryId,
    },

    #[serde(rename = "revision")]
    Revision {
        description: String,
        #[ts(as = "String")]
        base: git::Oid,
        #[ts(as = "String")]
        oid: git::Oid,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        #[ts(as = "Option<BTreeSet<(String, String)>>", optional)]
        resolves: BTreeSet<(cob::EntryId, cob::thread::CommentId)>,
    },
    #[serde(rename = "revision.edit")]
    RevisionEdit {
        #[ts(as = "String")]
        revision: patch::RevisionId,
        description: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(as = "Option<_>", optional)]
        embeds: Vec<cobs::thread::Embed>,
    },
    #[serde(rename = "revision.react")]
    RevisionReact {
        #[ts(as = "String")]
        revision: patch::RevisionId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        location: Option<cobs::thread::CodeLocation>,
        #[ts(as = "String")]
        reaction: cob::Reaction,
        active: bool,
    },
    #[serde(rename = "revision.redact")]
    RevisionRedact {
        #[ts(as = "String")]
        revision: patch::RevisionId,
    },
    #[serde(rename_all = "camelCase")]
    #[serde(rename = "revision.comment")]
    RevisionComment {
        #[ts(as = "String")]
        revision: patch::RevisionId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        location: Option<cobs::thread::CodeLocation>,
        body: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(as = "Option<String>", optional)]
        reply_to: Option<cob::thread::CommentId>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(as = "Option<_>", optional)]
        embeds: Vec<cobs::thread::Embed>,
    },
    #[serde(rename = "revision.comment.edit")]
    RevisionCommentEdit {
        #[ts(as = "String")]
        revision: patch::RevisionId,
        #[ts(as = "String")]
        comment: cob::thread::CommentId,
        body: String,
        #[ts(as = "Option<_>", optional)]
        embeds: Vec<cobs::thread::Embed>,
    },
    #[serde(rename = "revision.comment.redact")]
    RevisionCommentRedact {
        #[ts(as = "String")]
        revision: patch::RevisionId,
        #[ts(as = "String")]
        comment: cob::thread::CommentId,
    },
    #[serde(rename = "revision.comment.react")]
    RevisionCommentReact {
        #[ts(as = "String")]
        revision: patch::RevisionId,
        #[ts(as = "String")]
        comment: cob::thread::CommentId,
        #[ts(as = "String")]
        reaction: cob::Reaction,
        active: bool,
    },
}

impl FromRadicleAction<radicle::patch::Action> for Action {
    fn from_radicle_action(value: radicle::patch::Action, aliases: &Aliases) -> Self {
        match value {
            radicle::patch::Action::ReviewRedact { review } => Self::ReviewRedact { review },
            radicle::patch::Action::RevisionCommentReact {
                revision,
                comment,
                reaction,
                active,
            } => Self::RevisionCommentReact {
                revision,
                comment,
                reaction,
                active,
            },
            radicle::patch::Action::RevisionCommentRedact { revision, comment } => {
                Self::RevisionCommentRedact { revision, comment }
            }
            radicle::patch::Action::RevisionCommentEdit {
                revision,
                comment,
                body,
                embeds,
            } => Self::RevisionCommentEdit {
                revision,
                comment,
                body,
                embeds: embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
            },
            radicle::patch::Action::RevisionComment {
                revision,
                location,
                body,
                reply_to,
                embeds,
            } => Self::RevisionComment {
                revision,
                location: location.map(Into::into),
                body,
                reply_to,
                embeds: embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
            },
            radicle::patch::Action::RevisionRedact { revision } => {
                Self::RevisionRedact { revision }
            }
            radicle::patch::Action::RevisionReact {
                revision,
                location,
                reaction,
                active,
            } => Self::RevisionReact {
                revision,
                location: location.map(Into::into),
                reaction,
                active,
            },
            radicle::patch::Action::RevisionEdit {
                revision,
                description,
                embeds,
            } => Self::RevisionEdit {
                revision,
                description,
                embeds: embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
            },
            radicle::patch::Action::Assign { assignees } => Self::Assign {
                assignees: assignees
                    .iter()
                    .map(|a| Author::new(a, aliases))
                    .collect::<BTreeSet<_>>(),
            },
            radicle::patch::Action::Edit { title, target } => Self::Edit { title, target },
            radicle::patch::Action::Label { labels } => Self::Label { labels },
            radicle::patch::Action::Lifecycle { state } => Self::Lifecycle { state },
            radicle::patch::Action::Merge { revision, commit } => Self::Merge { revision, commit },
            radicle::patch::Action::Revision {
                description,
                base,
                oid,
                resolves,
            } => Self::Revision {
                description,
                base,
                oid,
                resolves,
            },

            radicle::patch::Action::Review {
                revision,
                summary,
                verdict,
                labels,
            } => Self::Review {
                revision,
                summary,
                verdict: verdict.map(Into::into),
                labels,
            },
            radicle::patch::Action::ReviewComment {
                review,
                body,
                location,
                reply_to,
                embeds,
            } => Self::ReviewComment {
                review,
                body,
                location: location.map(Into::into),
                reply_to,
                embeds: embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
            },
            radicle::patch::Action::ReviewCommentEdit {
                review,
                comment,
                body,
                embeds,
            } => Self::ReviewCommentEdit {
                review,
                comment,
                body,
                embeds: embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
            },
            radicle::patch::Action::ReviewCommentReact {
                review,
                comment,
                reaction,
                active,
            } => Self::ReviewCommentReact {
                review,
                comment,
                reaction,
                active,
            },
            radicle::patch::Action::ReviewCommentRedact { review, comment } => {
                Self::ReviewCommentRedact { review, comment }
            }
            radicle::patch::Action::ReviewCommentResolve { review, comment } => {
                Self::ReviewCommentResolve { review, comment }
            }
            radicle::patch::Action::ReviewCommentUnresolve { review, comment } => {
                Self::ReviewCommentUnresolve { review, comment }
            }
            radicle::patch::Action::ReviewEdit {
                review,
                summary,
                verdict,
                labels,
            } => Self::ReviewEdit {
                review,
                summary,
                verdict: verdict.map(Into::into),
                labels,
            },
        }
    }
}

#[derive(Debug, Default, TS, Serialize)]
#[ts(export)]
#[ts(export_to = "cob/patch/")]
#[serde(rename_all = "camelCase")]
pub struct PatchCounts {
    pub(crate) open: usize,
    pub(crate) draft: usize,
    pub(crate) archived: usize,
    pub(crate) merged: usize,
}

impl Index<Status> for PatchCounts {
    type Output = usize;

    fn index(&self, status: Status) -> &Self::Output {
        match status {
            Status::Draft => &self.draft,
            Status::Open => &self.open,
            Status::Archived => &self.archived,
            Status::Merged => &self.merged,
        }
    }
}

impl PatchCounts {
    pub fn new(open: usize, draft: usize, archived: usize, merged: usize) -> Self {
        Self {
            open,
            draft,
            archived,
            merged,
        }
    }

    pub fn total(&self) -> usize {
        self.open + self.draft + self.archived + self.merged
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CountsError {
    #[error(transparent)]
    Sqlite(#[from] sqlite::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    // to be extended as new error scenarios are introduced
}
