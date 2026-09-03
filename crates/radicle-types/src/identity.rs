use std::collections::BTreeMap;

use serde::Serialize;
use ts_rs::TS;

use radicle::cob::identity;
use radicle::identity::doc;
use radicle::node::AliasStore;

use crate::cobs::Author;
use crate::repo::Visibility;

/// The identity of a repository: the document as it stands now, plus every
/// revision that was ever proposed to it.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "identity/")]
pub struct Identity {
    #[ts(as = "String")]
    pub rid: radicle::identity::RepoId,
    /// Revision id of the document currently in force.
    #[ts(as = "String")]
    pub current: radicle::git::Oid,
    /// The document currently in force.
    pub doc: Doc,
    /// Every revision, newest first.
    pub revisions: Vec<Revision>,
}

/// A snapshot of an identity document.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "identity/")]
pub struct Doc {
    pub version: u32,
    pub delegates: Vec<Author>,
    /// Delegate signatures needed to make a ref canonical. This is *not* what
    /// governs changes to this document — see `majority`.
    pub threshold: usize,
    /// Delegate signatures needed to adopt a new revision of this document:
    /// a simple majority of the delegate set.
    pub majority: usize,
    pub visibility: Visibility,
    pub project: Option<Project>,
    /// Canonical-refs rules from the `xyz.radicle.crefs` payload, if any.
    pub canonical_refs: Vec<CanonicalRefRule>,
    /// Payload ids present on the document, including ones this app has no
    /// dedicated rendering for.
    pub payload_ids: Vec<String>,
    /// The document as stored, pretty-printed. Nothing above is hidden from
    /// this: it is the escape hatch for fields the UI does not model.
    pub raw: String,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "identity/")]
pub struct Project {
    pub name: String,
    pub description: String,
    pub default_branch: String,
}

/// One `refs/...` pattern and the rule that decides its canonical value.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "identity/")]
pub struct CanonicalRefRule {
    /// The ref pattern the rule applies to, e.g. `refs/tags/releases/*`.
    pub pattern: String,
    /// Who may sign off on this ref. Empty means the rule delegates to the
    /// document's delegate set rather than naming keys.
    pub allow: Vec<Author>,
    /// True when the rule defers to the document's delegates.
    pub delegates: bool,
    pub threshold: Option<usize>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "status")]
#[ts(export)]
#[ts(export_to = "identity/")]
pub enum State {
    /// Proposed and still gathering verdicts.
    Active,
    /// Adopted: it reached quorum among the delegates.
    Accepted,
    Rejected {
        reason: RejectedReason,
    },
    Redacted {
        reason: RedactedReason,
    },
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
#[ts(export_to = "identity/")]
pub enum RejectedReason {
    /// A majority of delegates rejected it.
    Vote,
    /// Its parent revision was rejected.
    Parent,
    /// A competing revision was accepted instead.
    Sibling {
        #[ts(as = "String")]
        revision: radicle::git::Oid,
    },
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
#[ts(export_to = "identity/")]
pub enum RedactedReason {
    /// Withdrawn by its author.
    Author,
    /// Its parent revision was redacted.
    Parent,
}

impl From<identity::State> for State {
    fn from(value: identity::State) -> Self {
        match value {
            identity::State::Active => State::Active,
            identity::State::Accepted => State::Accepted,
            identity::State::Rejected(by) => State::Rejected {
                reason: match by {
                    identity::RejectedBy::Vote => RejectedReason::Vote,
                    identity::RejectedBy::Parent => RejectedReason::Parent,
                    identity::RejectedBy::Sibling(id) => RejectedReason::Sibling { revision: id },
                },
            },
            identity::State::Redacted(by) => State::Redacted {
                reason: match by {
                    identity::RedactedBy::Author => RedactedReason::Author,
                    identity::RedactedBy::Parent => RedactedReason::Parent,
                },
            },
        }
    }
}

/// A proposed change to the identity document.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "identity/")]
pub struct Revision {
    #[ts(as = "String")]
    pub id: radicle::git::Oid,
    /// The document blob this revision points at.
    #[ts(as = "String")]
    pub blob: radicle::git::Oid,
    pub title: String,
    pub description: String,
    pub state: State,
    pub author: Author,
    #[ts(type = "number")]
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<String>", optional)]
    pub parent: Option<radicle::git::Oid>,
    /// Delegates that signed the revision.
    pub accepted: Vec<Author>,
    /// Delegates that voted against it.
    pub rejected: Vec<Author>,
    /// Whether the accepting signatures met the threshold in force.
    pub quorum: bool,
    /// The document as it would stand if this revision were adopted.
    pub doc: Doc,
    /// What this revision changes relative to its parent, in domain terms
    /// rather than as a text diff.
    pub changes: Vec<Change>,
}

/// A single field-level difference between a revision and its parent.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
#[ts(export_to = "identity/")]
pub enum Change {
    DelegateAdded {
        delegate: Author,
    },
    DelegateRemoved {
        delegate: Author,
    },
    ThresholdChanged {
        from: usize,
        to: usize,
    },
    VisibilityChanged {
        from: Visibility,
        to: Visibility,
    },
    NameChanged {
        from: String,
        to: String,
    },
    DescriptionChanged {
        from: String,
        to: String,
    },
    DefaultBranchChanged {
        from: String,
        to: String,
    },
    /// A payload this app does not model changed, was added or was removed.
    /// The raw document is the place to read the detail.
    PayloadChanged {
        payload: String,
        operation: PayloadOperation,
    },
}

#[derive(PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "identity/")]
pub enum PayloadOperation {
    Added,
    Removed,
    Updated,
}

impl Doc {
    pub fn new(doc: &doc::Doc, aliases: &impl AliasStore) -> Self {
        let payload_ids = doc
            .payload()
            .keys()
            .map(|id| id.to_string())
            .collect::<Vec<_>>();

        let project = doc.project().ok().map(|project| Project {
            name: project.name().to_string(),
            description: project.description().to_string(),
            default_branch: project.default_branch().to_string(),
        });

        Self {
            version: doc.version().number().get(),
            delegates: doc
                .delegates()
                .iter()
                .map(|did| Author::new(did, aliases))
                .collect(),
            threshold: doc.threshold(),
            majority: doc.majority(),
            visibility: match doc.visibility().clone() {
                radicle::identity::Visibility::Public => Visibility::Public,
                radicle::identity::Visibility::Private { allow } => Visibility::Private {
                    allow: allow.iter().map(|did| Author::new(did, aliases)).collect(),
                },
            },
            project,
            canonical_refs: canonical_ref_rules(doc, aliases),
            payload_ids,
            raw: raw_doc(doc),
        }
    }
}

/// The document as canonical JSON, pretty-printed for display. Falls back to
/// an empty string rather than failing the whole view if it cannot be encoded.
fn raw_doc(doc: &doc::Doc) -> String {
    serde_json::to_value(doc)
        .ok()
        .and_then(|value| serde_json::to_string_pretty(&value).ok())
        .unwrap_or_default()
}

/// Read the `xyz.radicle.crefs` payload straight off the document as JSON.
/// The typed `CanonicalRefs` requires validation that a historical revision
/// may not pass, and this view has to render those revisions anyway.
fn canonical_ref_rules(doc: &doc::Doc, aliases: &impl AliasStore) -> Vec<CanonicalRefRule> {
    let Some(payload) = doc.payload().get(&doc::PayloadId::canonical_refs()) else {
        return Vec::new();
    };
    let Some(rules) = payload.get("rules").and_then(|rules| rules.as_object()) else {
        return Vec::new();
    };

    rules
        .iter()
        .map(|(pattern, rule)| {
            let allow = rule
                .get("allow")
                .and_then(|allow| allow.as_array())
                .map(|allow| {
                    allow
                        .iter()
                        .filter_map(|did| did.as_str())
                        .filter_map(|did| did.parse::<radicle::identity::Did>().ok())
                        .map(|did| Author::new(&did, aliases))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            CanonicalRefRule {
                pattern: pattern.clone(),
                allow,
                delegates: rule
                    .get("delegates")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false),
                threshold: rule
                    .get("threshold")
                    .and_then(|value| value.as_u64())
                    .map(|value| value as usize),
            }
        })
        .collect()
}

/// Diff two documents field by field, so the UI can say "added delegate X"
/// instead of showing a JSON patch.
pub fn changes(from: Option<&doc::Doc>, to: &doc::Doc, aliases: &impl AliasStore) -> Vec<Change> {
    let Some(from) = from else {
        return Vec::new();
    };
    let mut changes = Vec::new();

    for did in to.delegates().iter() {
        if !from.delegates().contains(did) {
            changes.push(Change::DelegateAdded {
                delegate: Author::new(did, aliases),
            });
        }
    }
    for did in from.delegates().iter() {
        if !to.delegates().contains(did) {
            changes.push(Change::DelegateRemoved {
                delegate: Author::new(did, aliases),
            });
        }
    }

    if from.threshold() != to.threshold() {
        changes.push(Change::ThresholdChanged {
            from: from.threshold(),
            to: to.threshold(),
        });
    }

    if from.visibility() != to.visibility() {
        let convert = |visibility: &radicle::identity::Visibility| match visibility.clone() {
            radicle::identity::Visibility::Public => Visibility::Public,
            radicle::identity::Visibility::Private { allow } => Visibility::Private {
                allow: allow.iter().map(|did| Author::new(did, aliases)).collect(),
            },
        };
        changes.push(Change::VisibilityChanged {
            from: convert(from.visibility()),
            to: convert(to.visibility()),
        });
    }

    if let (Ok(before), Ok(after)) = (from.project(), to.project()) {
        if before.name() != after.name() {
            changes.push(Change::NameChanged {
                from: before.name().to_string(),
                to: after.name().to_string(),
            });
        }
        if before.description() != after.description() {
            changes.push(Change::DescriptionChanged {
                from: before.description().to_string(),
                to: after.description().to_string(),
            });
        }
        if before.default_branch() != after.default_branch() {
            changes.push(Change::DefaultBranchChanged {
                from: before.default_branch().to_string(),
                to: after.default_branch().to_string(),
            });
        }
    }

    changes.extend(payload_changes(from, to));

    changes
}

/// Compare payloads the typed fields above do not cover, so a change to e.g.
/// `xyz.radicle.crefs` is still reported rather than silently dropped.
fn payload_changes(from: &doc::Doc, to: &doc::Doc) -> Vec<Change> {
    let project = doc::PayloadId::project();
    let before: BTreeMap<_, _> = from
        .payload()
        .iter()
        .filter(|(id, _)| **id != project)
        .collect();
    let after: BTreeMap<_, _> = to
        .payload()
        .iter()
        .filter(|(id, _)| **id != project)
        .collect();

    let mut changes = Vec::new();
    for (id, payload) in &after {
        let operation = match before.get(id) {
            None => PayloadOperation::Added,
            Some(previous) if previous != payload => PayloadOperation::Updated,
            Some(_) => continue,
        };
        changes.push(Change::PayloadChanged {
            payload: id.to_string(),
            operation,
        });
    }
    for id in before.keys() {
        if !after.contains_key(id) {
            changes.push(Change::PayloadChanged {
                payload: id.to_string(),
                operation: PayloadOperation::Removed,
            });
        }
    }
    changes
}
