use radicle::cob::Label;
use radicle::identity;
use radicle::issue;
use radicle::node::{Alias, AliasStore};
use radicle::patch;
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
pub struct Issue {
    #[ts(as = "String")]
    id: String,
    author: Author,
    title: String,
    #[ts(type = "{ status: 'closed', reason: 'other' | 'solved' } | { status: 'open' } ")]
    state: issue::State,
    assignees: Vec<Author>,
    #[ts(as = "Vec<String>")]
    labels: Vec<Label>,
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
        }
    }
}

#[derive(TS, Serialize)]
#[ts(export)]
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
    labels: Vec<Label>,
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
        }
    }
}
