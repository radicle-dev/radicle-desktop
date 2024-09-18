use radicle::cob;
use radicle::git;
use radicle::identity;
use radicle::issue;
use radicle::node::{Alias, AliasStore};
use radicle::patch;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, TS)]
pub(crate) struct Author {
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
