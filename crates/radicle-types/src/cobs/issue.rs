use std::collections::BTreeSet;

use radicle::git;
use radicle::node::AliasStore;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::cob;
use radicle::identity;
use radicle::issue;

use crate::cobs;

#[derive(TS, Serialize)]
#[ts(export)]
#[ts(export_to = "cob/issue/")]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    id: String,
    author: cobs::Author,
    title: String,
    state: cobs::issue::State,
    assignees: Vec<cobs::Author>,
    discussion: Vec<cobs::thread::Comment<cobs::Never>>,
    #[ts(as = "Vec<String>")]
    labels: Vec<cob::Label>,
    #[ts(type = "number")]
    timestamp: cob::Timestamp,
}

impl Issue {
    pub fn new(id: &issue::IssueId, issue: &issue::Issue, aliases: &impl AliasStore) -> Self {
        Self {
            id: id.to_string(),
            author: cobs::Author::new(*issue.author().id(), aliases),
            title: issue.title().to_string(),
            state: (*issue.state()).into(),
            assignees: issue
                .assignees()
                .map(|did| cobs::Author::new(*did, aliases))
                .collect::<Vec<_>>(),
            discussion: issue
                .comments()
                .map(|(id, c)| cobs::thread::Comment::<cobs::Never>::new(*id, c.clone(), aliases))
                .collect::<Vec<_>>(),
            labels: issue.labels().cloned().collect::<Vec<_>>(),
            timestamp: issue.timestamp(),
        }
    }
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/issue/")]
pub struct Operation {
    #[ts(as = "String")]
    pub entry_id: git::Oid,
    #[ts(flatten)]
    pub action: Action,
    #[ts(type = "number")]
    pub timestamp: cob::Timestamp,
    pub author: cobs::Author,
}

#[derive(Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", tag = "status")]
#[ts(export)]
#[ts(export_to = "cob/issue/")]
pub enum State {
    Closed {
        reason: CloseReason,
    },
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
    pub embeds: Vec<cobs::thread::Embed>,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/issue/")]
pub enum Action {
    #[serde(rename = "assign")]
    Assign {
        #[ts(as = "Vec<String>")]
        assignees: BTreeSet<identity::Did>,
    },

    #[serde(rename = "edit")]
    Edit { title: String },

    #[serde(rename = "lifecycle")]
    Lifecycle { state: cobs::issue::State },

    #[serde(rename = "label")]
    Label {
        #[ts(as = "Vec<String>")]
        labels: BTreeSet<cob::Label>,
    },

    #[serde(rename_all = "camelCase")]
    #[serde(rename = "comment")]
    Comment {
        body: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(as = "Option<String>", optional)]
        reply_to: Option<cob::thread::CommentId>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(as = "Option<_>", optional)]
        embeds: Vec<cobs::thread::Embed>,
    },

    #[serde(rename = "comment.edit")]
    CommentEdit {
        #[ts(as = "String")]
        id: cob::thread::CommentId,
        body: String,
        #[ts(as = "Option<_>", optional)]
        embeds: Vec<cobs::thread::Embed>,
    },

    #[serde(rename = "comment.redact")]
    CommentRedact {
        #[ts(as = "String")]
        id: cob::thread::CommentId,
    },

    #[serde(rename = "comment.react")]
    CommentReact {
        #[ts(as = "String")]
        id: cob::thread::CommentId,
        #[ts(as = "String")]
        reaction: cob::Reaction,
        active: bool,
    },
}
