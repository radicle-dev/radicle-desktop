use std::collections::BTreeMap;
use std::collections::BTreeSet;

use radicle::node::AliasStore;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::cob;
use radicle::git;
use radicle::identity;
use radicle::patch;

use crate::cobs;

#[derive(TS, Serialize)]
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

impl Patch {
    pub fn new(id: patch::PatchId, patch: &patch::Patch, aliases: &impl AliasStore) -> Self {
        Self {
            id: id.to_string(),
            author: cobs::Author::new(*patch.author().id(), aliases),
            title: patch.title().to_string(),
            state: patch.state().clone().into(),
            base: *patch.base(),
            head: *patch.head(),
            assignees: patch
                .assignees()
                .map(|did| cobs::Author::new(did, aliases))
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

#[derive(Serialize, Deserialize, TS)]
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
    #[ts(as = "Option<String>", optional)]
    pub verdict: Option<cob::patch::Verdict>,
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
            author: cobs::Author::new(*value.author().id(), aliases),
            description: value
                .edits()
                .map(|e| Edit::new(e, aliases))
                .collect::<Vec<_>>(),
            base: *value.base(),
            head: value.head(),
            reviews: value
                .reviews()
                .map(|(id, r)| Review::new(*id, r.clone(), aliases))
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
                                authors,
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
            author: cobs::Author::new(edit.author.into(), aliases),
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
    id: identity::PublicKey,
    author: cobs::Author,
    #[ts(type = "'accept' | 'reject'")]
    #[ts(optional)]
    verdict: Option<cob::patch::Verdict>,
    #[ts(optional)]
    summary: Option<String>,
    #[ts(as = "Option<_>", optional)]
    comments: Vec<cobs::thread::Comment<cobs::thread::CodeLocation>>,
    #[ts(type = "number")]
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
            author: cobs::Author::new(review.author().id, aliases),
            verdict: review.verdict(),
            summary: review.summary().map(|s| s.to_string()),
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

