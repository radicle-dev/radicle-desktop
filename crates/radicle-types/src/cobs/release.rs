use std::collections::BTreeMap;

use radicle::identity::Did;
use radicle::node::AliasStore;
use radicle_artifact_core::cid as cid_utils;
use radicle_artifact_core::keys::EndpointId;
use serde::Serialize;
use ts_rs::TS;
use url::Url;

use crate::cobs;

#[derive(Clone, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/release/")]
pub struct Release {
    #[ts(as = "String")]
    pub id: radicle_artifact::ReleaseId,
    #[ts(as = "String")]
    pub oid: radicle::git::Oid,
    /// OID of the annotated tag that was recorded alongside the commit,
    /// if any. `None` means the release was created from a bare commit.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<String>", optional)]
    pub tag: Option<radicle::git::Oid>,
    /// Refname of the tag pointed at by `tag`, resolved on the server so
    /// the UI doesn't need to fan out a separate `list_tags` lookup per
    /// release. Omitted when the tag has been deleted locally.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag_name: Option<String>,
    /// DID of the user who authored this release COB.
    pub creator: cobs::Author,
    /// Subject line of the released commit, resolved on the server so the
    /// list view can render it without an extra round-trip per row.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub commit_summary: Option<String>,
    #[ts(type = "number")]
    pub timestamp: u64,
    pub artifacts: Vec<Artifact>,
}

#[derive(Clone, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/release/")]
pub struct Artifact {
    pub cid: String,
    pub name: String,
    pub kind: ArtifactKind,
    pub author: cobs::Author,
    pub locations: Vec<Location>,
    pub attestations: Vec<cobs::Author>,
    pub redactions: Vec<Redaction>,
    /// True when this device's iroh endpoint id appears as the host of
    /// one of the location URLs we (the local DID) wrote — i.e. we are
    /// actively advertising the artifact from the running process.
    pub seeded_from_here: bool,
    /// True when our DID has at least one `iroh://` URL on the COB whose
    /// host is *not* this device's iroh endpoint. Usually means we (or a
    /// past install on a different machine, e.g. the CLI) advertised this
    /// artifact from somewhere else. The bytes are not necessarily local.
    pub seeded_from_other: bool,
    /// Free-form key/value annotations contributed by the artifact author
    /// or repo delegates. Authorization is enforced upstream.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(type = "Record<string, unknown>", optional)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/release/")]
pub enum ArtifactKind {
    Blob,
    Collection,
    Unknown,
}

#[derive(Clone, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/release/")]
pub struct Location {
    pub peer: cobs::Author,
    pub urls: Vec<String>,
}

#[derive(Clone, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "cob/release/")]
pub struct Redaction {
    pub peer: cobs::Author,
    pub reason: String,
}

impl Release {
    pub fn new(
        id: radicle_artifact::ReleaseId,
        release: &radicle_artifact::Release,
        aliases: &impl AliasStore,
        tag_name: Option<String>,
        commit_summary: Option<String>,
    ) -> Self {
        let artifacts = release
            .artifacts()
            .iter()
            .map(|(cid, artifact)| Artifact::new(cid, artifact, aliases))
            .collect();

        Self {
            id,
            oid: *release.oid(),
            tag: release.tag().copied(),
            tag_name,
            creator: cobs::Author::new(release.creator(), aliases),
            commit_summary,
            timestamp: release.timestamp(),
            artifacts,
        }
    }
}

impl Artifact {
    pub fn new(
        cid: &radicle_artifact::Cid,
        artifact: &radicle_artifact::Artifact,
        aliases: &impl AliasStore,
    ) -> Self {
        let kind = match cid_utils::artifact_kind(cid) {
            Ok(cid_utils::ArtifactKind::Blob) => ArtifactKind::Blob,
            Ok(cid_utils::ArtifactKind::Collection) => ArtifactKind::Collection,
            Err(_) => ArtifactKind::Unknown,
        };

        let locations = artifact
            .locations()
            .iter()
            .map(|(did, urls)| Location {
                peer: cobs::Author::new(did, aliases),
                urls: urls.iter().map(|u| u.to_string()).collect(),
            })
            .collect();

        let attestations = artifact
            .attestations()
            .iter()
            .map(|did| cobs::Author::new(did, aliases))
            .collect();

        let redactions = artifact
            .redactions()
            .iter()
            .map(|(did, reason)| Redaction {
                peer: cobs::Author::new(did, aliases),
                reason: reason.clone(),
            })
            .collect();

        // We don't yet know which iroh endpoint id is "us-here" — that
        // lives in the iroh state, not in this crate. Default both flags
        // to false and let `set_endpoint_flags` refine them at the
        // driver layer when an endpoint id is available.
        Self {
            cid: cid.to_string(),
            name: artifact.name().to_string(),
            kind,
            author: cobs::Author::new(artifact.author(), aliases),
            locations,
            attestations,
            redactions,
            seeded_from_here: false,
            seeded_from_other: false,
            metadata: artifact.metadata().clone(),
        }
    }
}

/// Classify the `radiroh://` URLs under our DID into `seeded_from_here`
/// (endpoint matches `our_endpoint`) and `seeded_from_other` (any other
/// endpoint). A bare `radiroh://` under our DID derives to our own
/// endpoint, so it counts as "here". Drivers call this on every release
/// they hand to the UI so the flags reflect *this* identity, not just
/// "we wrote a location once". Legacy `iroh://` URLs are ignored.
pub fn set_endpoint_flags(release: &mut Release, our_did: &Did, our_endpoint: &EndpointId) {
    let our_did_str = our_did.to_string();
    for artifact in &mut release.artifacts {
        let mut here = false;
        let mut other = false;
        for loc in &artifact.locations {
            if loc.peer.did().to_string() != our_did_str {
                continue;
            }
            for url in &loc.urls {
                let Ok(url) = Url::parse(url) else {
                    continue;
                };
                if !EndpointId::is_endpoint_url(&url) {
                    continue;
                }
                match EndpointId::from_url(&url) {
                    // Explicit endpoint host: compare against ours.
                    Ok(Some(id)) if &id == our_endpoint => here = true,
                    Ok(Some(_)) => other = true,
                    // Bare `radiroh://` derives to our DID's endpoint.
                    Ok(None) => here = true,
                    Err(_) => {}
                }
            }
        }
        artifact.seeded_from_here = here;
        artifact.seeded_from_other = other;
    }
}

// Quiet the dead-code warning: this map alias keeps the upstream type names
// aligned with the DTO at a glance, and may be used later by trait helpers.
#[allow(dead_code)]
pub(crate) type LocationMap = BTreeMap<Did, std::collections::BTreeSet<url::Url>>;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use std::collections::{BTreeMap, HashMap};
    use std::str::FromStr;

    use radicle::crypto::Signer;
    use radicle::crypto::test::signer::MockSigner;
    use radicle::identity::Did;
    use radicle::node::{Alias, NodeId};
    use radicle_artifact::ReleaseId;
    use radicle_artifact_core::keys::EndpointId;

    use super::{Artifact, ArtifactKind, Location, Release, set_endpoint_flags};
    use crate::cobs;

    // A deterministic DID derived from a seed. The 32 key bytes double as a
    // valid iroh endpoint id, so `EndpointId::try_from` always succeeds.
    fn did(seed: u8) -> Did {
        Did::from(*MockSigner::from_seed([seed; 32]).public_key())
    }

    // `set_endpoint_flags` only reads the peer DID, so an empty alias store
    // (`HashMap` implements `AliasStore`) is enough to build an `Author`.
    fn author(did: &Did) -> cobs::Author {
        cobs::Author::new(did, &HashMap::<NodeId, Alias>::new())
    }

    fn location(peer: &Did, urls: &[&str]) -> Location {
        Location {
            peer: author(peer),
            urls: urls.iter().map(|u| u.to_string()).collect(),
        }
    }

    // A single-artifact release whose one artifact carries `locations`. The
    // id/oid/timestamp are unused by `set_endpoint_flags` but must be valid.
    fn release_with_locations(locations: Vec<Location>) -> Release {
        let oid = radicle::git::Oid::from_str("0123456789012345678901234567890123456789").unwrap();
        Release {
            id: ReleaseId::from_str("0123456789012345678901234567890123456789").unwrap(),
            oid,
            tag: None,
            tag_name: None,
            creator: author(&did(9)),
            commit_summary: None,
            timestamp: 0,
            artifacts: vec![Artifact {
                cid: "test-cid".to_string(),
                name: "test".to_string(),
                kind: ArtifactKind::Blob,
                author: author(&did(9)),
                locations,
                attestations: vec![],
                redactions: vec![],
                seeded_from_here: false,
                seeded_from_other: false,
                metadata: BTreeMap::new(),
            }],
        }
    }

    #[test]
    fn our_endpoint_url_is_here() {
        let our_did = did(1);
        let our_endpoint = EndpointId::try_from(&our_did).unwrap();
        let url = our_endpoint.to_url().to_string();

        let mut release = release_with_locations(vec![location(&our_did, &[&url])]);
        set_endpoint_flags(&mut release, &our_did, &our_endpoint);

        assert!(release.artifacts[0].seeded_from_here);
        assert!(!release.artifacts[0].seeded_from_other);
    }

    #[test]
    fn other_endpoint_url_is_other() {
        let our_did = did(1);
        let our_endpoint = EndpointId::try_from(&our_did).unwrap();
        // Authored by our DID but pointing at a different endpoint — e.g. a
        // past install of ours on another machine.
        let url = EndpointId::try_from(&did(2)).unwrap().to_url().to_string();

        let mut release = release_with_locations(vec![location(&our_did, &[&url])]);
        set_endpoint_flags(&mut release, &our_did, &our_endpoint);

        assert!(!release.artifacts[0].seeded_from_here);
        assert!(release.artifacts[0].seeded_from_other);
    }

    #[test]
    fn bare_radiroh_url_is_here() {
        let our_did = did(1);
        let our_endpoint = EndpointId::try_from(&our_did).unwrap();

        // A bare `radiroh://` under our DID derives to our own endpoint.
        let mut release = release_with_locations(vec![location(&our_did, &["radiroh://"])]);
        set_endpoint_flags(&mut release, &our_did, &our_endpoint);

        assert!(release.artifacts[0].seeded_from_here);
        assert!(!release.artifacts[0].seeded_from_other);
    }

    #[test]
    fn legacy_and_non_endpoint_urls_are_ignored() {
        let our_did = did(1);
        let our_endpoint = EndpointId::try_from(&our_did).unwrap();

        let mut release = release_with_locations(vec![location(
            &our_did,
            &["iroh://legacyhost", "https://example.com/artifact"],
        )]);
        set_endpoint_flags(&mut release, &our_did, &our_endpoint);

        assert!(!release.artifacts[0].seeded_from_here);
        assert!(!release.artifacts[0].seeded_from_other);
    }

    #[test]
    fn locations_under_other_dids_are_ignored() {
        let our_did = did(1);
        let our_endpoint = EndpointId::try_from(&our_did).unwrap();
        // A peer advertises our endpoint URL under *their* DID; that's them
        // hosting it, not us.
        let url = our_endpoint.to_url().to_string();

        let mut release = release_with_locations(vec![location(&did(2), &[&url])]);
        set_endpoint_flags(&mut release, &our_did, &our_endpoint);

        assert!(!release.artifacts[0].seeded_from_here);
        assert!(!release.artifacts[0].seeded_from_other);
    }

    #[test]
    fn here_and_other_can_both_be_set() {
        let our_did = did(1);
        let our_endpoint = EndpointId::try_from(&our_did).unwrap();
        let our_url = our_endpoint.to_url().to_string();
        let other_url = EndpointId::try_from(&did(2)).unwrap().to_url().to_string();

        let mut release = release_with_locations(vec![location(&our_did, &[&our_url, &other_url])]);
        set_endpoint_flags(&mut release, &our_did, &our_endpoint);

        assert!(release.artifacts[0].seeded_from_here);
        assert!(release.artifacts[0].seeded_from_other);
    }

    #[test]
    fn no_locations_leaves_flags_false() {
        let our_did = did(1);
        let our_endpoint = EndpointId::try_from(&our_did).unwrap();

        let mut release = release_with_locations(vec![]);
        set_endpoint_flags(&mut release, &our_did, &our_endpoint);

        assert!(!release.artifacts[0].seeded_from_here);
        assert!(!release.artifacts[0].seeded_from_other);
    }
}
