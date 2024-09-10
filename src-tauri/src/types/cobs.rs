use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::cob;
use radicle::crypto;
use radicle::identity;
use radicle::issue;
use radicle::node::{Alias, AliasStore};
use radicle::patch;
use radicle::storage::git;

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
    discussion: Vec<Comment>,
    #[ts(as = "Vec<String>")]
    labels: Vec<cob::Label>,
    #[ts(type = "number")]
    timestamp: cob::Timestamp,
}

impl Issue {
    pub fn new(id: issue::IssueId, issue: issue::Issue, aliases: &impl AliasStore) -> Self {
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
                .map(|(id, c)| Comment::<Never>::new(*id, c.clone(), aliases))
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
            assignees: patch
                .assignees()
                .map(|did| Author::new(did, aliases))
                .collect::<Vec<_>>(),
            labels: patch.labels().cloned().collect::<Vec<_>>(),
            timestamp: patch.timestamp(),
            revision_count: patch.revisions().count(),
        }
    }
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Revision {
    author: Author,
    description: Vec<Edit>,
    #[ts(as = "String")]
    base: git::Oid,
    #[ts(as = "String")]
    oid: git::Oid,
    reviews: Vec<Review>,
    #[ts(as = "i64")]
    timestamp: cob::common::Timestamp,
    discussion: Vec<Comment<CodeLocation>>,
    reactions: Vec<Reaction>,
}

impl Revision {
    pub fn new(value: cob::patch::Revision, aliases: &impl AliasStore) -> Self {
        Self {
            author: Author::new(*value.author().id(), aliases),
            description: value
                .edits()
                .map(|e| Edit::new(e, aliases))
                .collect::<Vec<_>>(),
            base: *value.base(),
            oid: value.head(),
            reviews: value
                .reviews()
                .map(|(id, r)| Review::new(*id, r.clone(), aliases))
                .collect::<Vec<_>>(),
            timestamp: value.timestamp(),
            discussion: value
                .discussion()
                .comments()
                .map(|(id, c)| Comment::<CodeLocation>::new(*id, c.clone(), aliases))
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
                                location.as_ref().map(|l| CodeLocation::new(l.clone())),
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
    location: Option<CodeLocation>,
}

impl Reaction {
    pub fn new(
        emoji: cob::Reaction,
        authors: Vec<&crypto::PublicKey>,
        location: Option<CodeLocation>,
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
    comments: Vec<Comment<CodeLocation>>,
    #[ts(as = "i64")]
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
                .map(|(id, c)| Comment::<CodeLocation>::new(*id, c.clone(), aliases))
                .collect::<Vec<_>>(),
            timestamp: review.timestamp(),
        }
    }
}

#[derive(TS, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    pub author: Author,
    #[ts(as = "i64")]
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

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Comment<T = Never> {
    #[ts(as = "String")]
    id: cob::thread::CommentId,
    author: Author,
    edits: Vec<Edit>,
    #[ts(as = "String")]
    reactions: BTreeMap<cob::common::Reaction, Vec<cob::op::ActorId>>,
    #[ts(as = "Option<String>")]
    #[ts(optional)]
    reply_to: Option<cob::thread::CommentId>,
    #[ts(optional)]
    location: Option<T>,
    resolved: bool,
}

impl Comment<CodeLocation> {
    pub fn new(
        id: cob::thread::CommentId,
        comment: cob::thread::Comment<cob::common::CodeLocation>,
        aliases: &impl AliasStore,
    ) -> Self {
        Self {
            id,
            author: Author::new(comment.author().into(), aliases),
            edits: comment
                .edits()
                .map(|e| Edit::new(e, aliases))
                .collect::<Vec<_>>(),
            reactions: comment
                .reactions()
                .into_iter()
                .map(|(r, a)| (*r, a.into_iter().copied().collect::<Vec<_>>()))
                .collect::<BTreeMap<cob::common::Reaction, Vec<cob::op::ActorId>>>(),
            reply_to: comment.reply_to(),
            location: comment.location().map(|l| CodeLocation::new(l.clone())),
            resolved: comment.is_resolved(),
        }
    }
}

impl Comment {
    pub fn new(
        id: cob::thread::CommentId,
        comment: cob::thread::Comment,
        aliases: &impl AliasStore,
    ) -> Self {
        Self {
            id,
            author: Author::new(comment.author().into(), aliases),
            edits: comment
                .edits()
                .map(|e| Edit::new(e, aliases))
                .collect::<Vec<_>>(),
            reactions: comment
                .reactions()
                .into_iter()
                .map(|(r, a)| (*r, a.into_iter().copied().collect::<Vec<_>>()))
                .collect::<BTreeMap<cob::common::Reaction, Vec<cob::op::ActorId>>>(),
            reply_to: comment.reply_to(),
            location: None,
            resolved: comment.is_resolved(),
        }
    }
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct NewPatchComment {
    pub id: String,
    pub revision: String,
    pub body: String,
    #[ts(as = "Option<String>")]
    #[ts(optional)]
    pub reply_to: Option<cob::thread::CommentId>,
    pub location: Option<CodeLocation>,
    #[ts(type = "{ name: string, content: string }[]")]
    pub embeds: Vec<cob::Embed>,
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct NewIssueComment {
    pub id: String,
    pub body: String,
    #[ts(as = "Option<String>")]
    #[ts(optional)]
    pub reply_to: Option<cob::thread::CommentId>,
    #[ts(type = "{ name: string, content: string }[]")]
    pub embeds: Vec<cob::Embed>,
}

#[derive(TS, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CodeLocation {
    #[ts(as = "String")]
    commit: git::Oid,
    path: std::path::PathBuf,
    old: Option<CodeRange>,
    new: Option<CodeRange>,
}

impl From<cob::CodeLocation> for CodeLocation {
    fn from(val: cob::CodeLocation) -> Self {
        Self {
            commit: val.commit,
            path: val.path,
            old: val.old.map(|o| o.into()),
            new: val.new.map(|o| o.into()),
        }
    }
}

impl CodeLocation {
    pub fn new(location: cob::common::CodeLocation) -> Self {
        Self {
            commit: location.commit,
            path: location.path,
            old: location.old.map(|l| l.into()),
            new: location.new.map(|l| l.into()),
        }
    }
}

impl From<CodeLocation> for cob::CodeLocation {
    fn from(val: CodeLocation) -> Self {
        Self {
            commit: val.commit,
            path: val.path,
            old: val.old.map(|o| o.into()),
            new: val.new.map(|o| o.into()),
        }
    }
}

#[derive(TS, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
pub enum CodeRange {
    Lines {
        #[ts(type = "{ start: number, end: number }")]
        range: std::ops::Range<usize>,
    },
    Chars {
        line: usize,
        #[ts(type = "{ start: number, end: number }")]
        range: std::ops::Range<usize>,
    },
}

impl From<cob::CodeRange> for CodeRange {
    fn from(val: cob::CodeRange) -> Self {
        match val {
            cob::CodeRange::Chars { line, range } => Self::Chars { line, range },
            cob::CodeRange::Lines { range } => Self::Lines { range },
        }
    }
}

impl From<CodeRange> for cob::CodeRange {
    fn from(val: CodeRange) -> Self {
        match val {
            CodeRange::Chars { line, range } => Self::Chars { line, range },
            CodeRange::Lines { range } => Self::Lines { range },
        }
    }
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
