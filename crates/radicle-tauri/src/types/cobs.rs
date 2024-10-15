use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::cob;
use radicle::crypto;
use radicle::identity;
use radicle::node::{Alias, AliasStore};

pub mod issue;
pub mod patch;

use crate::types::thread;

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

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/")]
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

/// The `Infallible` type does not have a `Serialize`/`Deserialize`
/// implementation. The `Never` type imitates `Infallible` and
/// provides the derived implementations.
#[derive(TS, Serialize)]
#[ts(export)]
#[ts(export_to = "utils/")]
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
    pub embeds: Vec<cob::Embed<cob::Uri>>,
}

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
