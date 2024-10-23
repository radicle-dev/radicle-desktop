use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::identity;
use radicle::node::{Alias, AliasStore};

pub mod issue;
pub mod patch;
pub mod thread;

#[derive(Serialize, TS)]
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
    pub fn new(did: identity::Did, aliases: &impl AliasStore) -> Self {
        Self {
            did,
            alias: aliases.alias(&did),
        }
    }
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

#[derive(Serialize, Deserialize, TS)]
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
