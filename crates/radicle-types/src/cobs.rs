use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::identity;
use radicle::node::{Alias, AliasStore};

pub mod diff;
pub mod issue;
pub mod patch;
pub mod stream;
pub mod thread;

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/")]
pub struct Author {
    #[ts(as = "String")]
    did: identity::Did,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<String>", optional)]
    alias: Option<Alias>,
}

impl Author {
    pub fn new(did: &identity::Did, aliases: &impl AliasStore) -> Self {
        Self {
            did: *did,
            alias: aliases.alias(did),
        }
    }
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/")]
pub struct EmbedWithMimeType {
    pub content: Vec<u8>,
    pub mime_type: Option<String>,
}

#[derive(TS, Serialize)]
#[doc = "A type alias for the TS type `never`."]
#[ts(export)]
#[ts(export_to = "cob/")]
pub enum Never {}

#[derive(TS, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/")]
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

#[derive(Serialize, Deserialize, TS, Debug, PartialEq, Clone)]
#[ts(export)]
#[ts(export_to = "cob/")]
pub struct PaginatedQuery<T> {
    pub cursor: usize,
    pub more: bool,
    pub content: T,
}

#[derive(TS, Serialize)]
#[ts(export)]
#[ts(export_to = "cob/")]
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

pub mod query {
    use serde::{Deserialize, Serialize};

    use radicle::issue;
    use radicle::patch;

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum IssueStatus {
        Closed,
        #[default]
        Open,
        All,
    }

    impl IssueStatus {
        pub fn matches(&self, issue: &issue::State) -> bool {
            match self {
                Self::Open => matches!(issue, issue::State::Open),
                Self::Closed => matches!(issue, issue::State::Closed { .. }),
                Self::All => true,
            }
        }
    }

    #[derive(Default, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub enum PatchStatus {
        #[default]
        Open,
        Draft,
        Archived,
        Merged,
    }

    impl From<patch::Status> for PatchStatus {
        fn from(value: patch::Status) -> Self {
            match value {
                patch::Status::Archived => Self::Archived,
                patch::Status::Draft => Self::Draft,
                patch::Status::Merged => Self::Merged,
                patch::Status::Open => Self::Open,
            }
        }
    }
    impl From<PatchStatus> for patch::Status {
        fn from(value: PatchStatus) -> Self {
            match value {
                PatchStatus::Archived => Self::Archived,
                PatchStatus::Draft => Self::Draft,
                PatchStatus::Merged => Self::Merged,
                PatchStatus::Open => Self::Open,
            }
        }
    }
}
