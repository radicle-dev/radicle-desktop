use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::node::AliasStore;
use radicle::{cob, git};

use crate::types::cobs;
use crate::types::thread;

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CreateReviewComment {
    #[ts(as = "String")]
    pub review_id: cob::patch::ReviewId,
    pub body: String,
    #[ts(as = "Option<String>")]
    pub reply_to: Option<cob::thread::CommentId>,
    pub location: Option<thread::CodeLocation>,
    #[ts(type = "{ name: string, content: string }[]")]
    pub embeds: Vec<cob::Embed<cob::Uri>>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Comment<T> {
    #[ts(as = "String")]
    id: cob::thread::CommentId,
    author: cobs::Author,
    edits: Vec<cobs::Edit>,
    reactions: Vec<cobs::Reaction>,
    #[ts(as = "Option<String>")]
    #[ts(optional)]
    reply_to: Option<cob::thread::CommentId>,
    #[ts(optional)]
    location: Option<T>,
    #[ts(type = "{ name: string, content: string }[]")]
    embeds: Vec<cob::Embed<cob::Uri>>,
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
            author: cobs::Author::new(comment.author().into(), aliases),
            edits: comment
                .edits()
                .map(|e| cobs::Edit::new(e, aliases))
                .collect::<Vec<_>>(),
            reactions: comment
                .reactions()
                .into_iter()
                .map(|(reaction, authors)| cobs::Reaction::new(*reaction, authors, None, aliases))
                .collect::<Vec<_>>(),
            reply_to: comment.reply_to(),
            location: comment.location().map(|l| CodeLocation::new(l.clone())),
            embeds: comment.embeds().to_vec(),
            resolved: comment.is_resolved(),
        }
    }
}

impl Comment<cobs::Never> {
    pub fn new(
        id: cob::thread::CommentId,
        comment: cob::thread::Comment,
        aliases: &impl AliasStore,
    ) -> Self {
        Self {
            id,
            author: cobs::Author::new(comment.author().into(), aliases),
            edits: comment
                .edits()
                .map(|e| cobs::Edit::new(e, aliases))
                .collect::<Vec<_>>(),
            reactions: comment
                .reactions()
                .into_iter()
                .map(|(reaction, authors)| cobs::Reaction::new(*reaction, authors, None, aliases))
                .collect::<Vec<_>>(),
            reply_to: comment.reply_to(),
            location: None,
            embeds: comment.embeds().to_vec(),
            resolved: comment.is_resolved(),
        }
    }
}

#[derive(Clone, TS, Serialize, Deserialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct NewIssueComment {
    #[ts(as = "String")]
    pub id: git::Oid,
    pub body: String,
    #[ts(as = "Option<String>")]
    #[ts(optional)]
    pub reply_to: Option<cob::thread::CommentId>,
    #[ts(type = "{ name: string, content: string }[]")]
    pub embeds: Vec<cob::Embed<cob::Uri>>,
}

#[derive(Clone, TS, Serialize, Deserialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct NewPatchComment {
    #[ts(as = "String")]
    pub id: git::Oid,
    #[ts(as = "String")]
    pub revision: git::Oid,
    pub body: String,
    #[ts(as = "Option<String>")]
    #[ts(optional)]
    pub reply_to: Option<cob::thread::CommentId>,
    pub location: Option<CodeLocation>,
    #[ts(type = "{ name: string, content: string }[]")]
    pub embeds: Vec<cob::Embed<cob::Uri>>,
}

#[derive(Clone, TS, Serialize, Deserialize)]
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

#[derive(Clone, TS, Serialize, Deserialize)]
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
