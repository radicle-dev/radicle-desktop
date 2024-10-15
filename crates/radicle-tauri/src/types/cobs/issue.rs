use std::collections::BTreeSet;

use radicle::git;
use radicle::node::AliasStore;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::cob;
use radicle::identity;
use radicle::issue;

use crate::types::cobs::{self, Author, Never};
use crate::types::thread;

#[derive(TS, Serialize)]
#[ts(export)]
#[ts(export_to = "cob/issue/")]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    id: String,
    author: Author,
    title: String,
    state: cobs::issue::State,
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
            state: (*issue.state()).into(),
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

#[derive(Serialize, TS)]
#[ts(export)]
#[ts(export_to = "cob/issue/")]
pub struct IssueOps {
    #[ts(as = "String")]
    id: git::Oid,
    action: Action,
    #[ts(type = "number")]
    timestamp: cob::Timestamp,
    author: Author,
}

#[derive(Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", tag = "status")]
#[ts(export)]
#[ts(export_to = "cob/issue/")]
pub enum State {
    /// The issue is closed.
    Closed { reason: CloseReason },
    /// The issue is open.
    #[default]
    Open,
}

impl From<State> for issue::State {
    fn from(value: State) -> Self {
        match value {
            State::Closed { reason } => Self::Closed {
                reason: reason.into(),
            },
            State::Open => Self::Open,
        }
    }
}

impl From<issue::State> for State {
    fn from(value: issue::State) -> Self {
        match value {
            issue::State::Closed { reason } => Self::Closed {
                reason: reason.into(),
            },
            issue::State::Open => Self::Open,
        }
    }
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/issue/")]
pub enum CloseReason {
    Other,
    Solved,
}

impl From<CloseReason> for issue::CloseReason {
    fn from(value: CloseReason) -> Self {
        match value {
            CloseReason::Other => Self::Other,
            CloseReason::Solved => Self::Solved,
        }
    }
}

impl From<issue::CloseReason> for CloseReason {
    fn from(value: issue::CloseReason) -> Self {
        match value {
            issue::CloseReason::Other => Self::Other,
            issue::CloseReason::Solved => Self::Solved,
        }
    }
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
#[ts(export_to = "cob/issue/")]
#[serde(rename_all = "camelCase")]
pub struct NewIssue {
    pub title: String,
    pub description: String,
    #[ts(as = "Option<Vec<String>>", optional)]
    pub labels: Vec<cob::Label>,
    #[ts(as = "Option<Vec<String>>", optional)]
    pub assignees: Vec<identity::Did>,
    #[ts(as = "Option<_>", optional)]
    pub embeds: Vec<thread::Embed>,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/issue/")]
pub enum Action {
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
    Lifecycle { state: cobs::issue::State },

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
        #[ts(as = "Option<String>", optional)]
        reply_to: Option<cob::thread::CommentId>,
        /// Embeded content.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(as = "Option<_>", optional)]
        embeds: Vec<thread::Embed>,
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
        #[ts(as = "Option<_>", optional)]
        embeds: Vec<thread::Embed>,
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
