use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::str::FromStr;

use radicle::identity;
use radicle::storage::{ReadRepository, ReadStorage};
use radicle_artifact::Releases as ReleasesStore;
use radicle_artifact_core::cid as cid_utils;
use radicle_artifact_core::keys::EndpointId;
use radicle_surf as surf;

use crate::cobs::release;
use crate::error::Error;
use crate::traits::Profile;

pub trait Releases: Profile {
    /// List every release for a repo, newest-first by COB timestamp.
    fn list_releases(&self, rid: identity::RepoId) -> Result<Vec<release::Release>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let aliases = profile.aliases();

        let surf_repo = surf::Repository::open(repo.path())?;
        // Build the tag-OID → refname index once so the list view can show
        // names without an N+1 ref walk per release.
        let tag_index = build_tag_index(&surf_repo);
        let releases = ReleasesStore::open(&repo)?;

        let mut out = Vec::new();
        for item in releases.all()? {
            let (id, release) = item?;
            let tag_name = release
                .tag()
                .and_then(|oid| tag_index.get(&oid.to_string()).cloned());
            let commit_summary = commit_summary(&surf_repo, *release.oid());
            out.push(release::Release::new(
                radicle_artifact::ReleaseId::from(id),
                &release,
                &aliases,
                tag_name,
                commit_summary,
            ));
        }
        // Newest first. Underlying iterator order isn't guaranteed across
        // radicle-artifact versions, so sort explicitly instead of just
        // reversing.
        out.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        apply_endpoint_flags(&profile, &mut out);
        Ok(out)
    }

    fn release_by_id(
        &self,
        rid: identity::RepoId,
        release_id: String,
    ) -> Result<Option<release::Release>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let aliases = profile.aliases();

        let id = radicle_artifact::ReleaseId::from_str(&release_id)?;
        let releases = ReleasesStore::open(&repo)?;
        let Some(release) = releases.get(&id)? else {
            return Ok(None);
        };

        let surf_repo = surf::Repository::open(repo.path())?;
        let tag_name = release
            .tag()
            .and_then(|oid| build_tag_index(&surf_repo).get(&oid.to_string()).cloned());
        let commit_summary = commit_summary(&surf_repo, *release.oid());

        let mut out = release::Release::new(id, &release, &aliases, tag_name, commit_summary);
        apply_endpoint_flags(&profile, std::slice::from_mut(&mut out));
        Ok(Some(out))
    }

    fn releases_by_commit(
        &self,
        rid: identity::RepoId,
        oid: radicle::git::Oid,
    ) -> Result<Vec<release::Release>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let aliases = profile.aliases();

        let surf_repo = surf::Repository::open(repo.path())?;
        let tag_index = build_tag_index(&surf_repo);
        let releases = ReleasesStore::open(&repo)?;

        let mut out = Vec::new();
        for item in releases.find_by_commit(oid)? {
            let (id, release) = item?;
            let tag_name = release
                .tag()
                .and_then(|tag_oid| tag_index.get(&tag_oid.to_string()).cloned());
            let commit_summary = commit_summary(&surf_repo, *release.oid());
            out.push(release::Release::new(
                id,
                &release,
                &aliases,
                tag_name,
                commit_summary,
            ));
        }
        apply_endpoint_flags(&profile, &mut out);
        Ok(out)
    }

    /// Compute the BLAKE3-derived CID for a local file or directory.
    /// Files become Blob CIDs, directories become Collection CIDs.
    fn compute_cid(&self, path: PathBuf) -> Result<String, Error> {
        let cid = if path.is_dir() {
            cid_utils::compute_content_id(&path)?
        } else {
            cid_utils::compute_blob_cid(&path)?
        };
        Ok(cid.to_string())
    }

    /// Snapshot the COB locations for a single artifact in a release. Used
    /// by the download command to build the location list before running
    /// the iroh-blobs Downloader; cloned out so the COB lock is released
    /// before the (long) async fetch begins.
    fn artifact_locations(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
    ) -> Result<BTreeMap<identity::Did, BTreeSet<url::Url>>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;

        let id = radicle_artifact::ReleaseId::from_str(&release_id)?;
        let parsed_cid = cid::Cid::from_str(&cid)?;

        let releases = ReleasesStore::open(&repo)?;
        let release = releases.get(&id)?.ok_or_else(|| Error::ReleaseNotFound {
            release_id: release_id.clone(),
        })?;
        let artifact =
            release
                .artifact(&parsed_cid)
                .ok_or_else(|| Error::ArtifactNotInRelease {
                    cid: cid.clone(),
                    release_id: release_id.clone(),
                })?;
        Ok(artifact.locations().clone())
    }
}

/// Build a lookup from tag-object OID to short refname so the list view
/// can show e.g. `v1.0.0` instead of a 40-char hash. Annotated tag OIDs
/// land under their tag-object OID; lightweight tags are keyed by the
/// commit they point at, matching `Release::tag`'s storage convention.
fn build_tag_index(repo: &surf::Repository) -> HashMap<String, String> {
    use surf::{Glob, Tag};

    let mut out = HashMap::new();
    let Ok(tags) = repo.tags(&Glob::all_tags()) else {
        return out;
    };
    for tag in tags.flatten() {
        let (id, name) = match tag {
            Tag::Light { id, name } => (id, name),
            Tag::Annotated { id, name, .. } => (id, name),
        };
        out.insert(id.to_string(), name.to_string());
    }
    out
}

/// Refine each release's per-artifact `seeded_from_here` / `seeded_from_other`
/// flags against our own iroh endpoint, derived from our DID. Applied at the
/// data layer so every driver returns consistent flags without each command
/// re-deriving the endpoint. Independent of the node, so listing works even
/// when it's down.
fn apply_endpoint_flags(profile: &radicle::Profile, releases: &mut [release::Release]) {
    let did = identity::Did::from(profile.public_key);
    let Ok(endpoint) = EndpointId::try_from(&did) else {
        return;
    };
    for release in releases {
        release::set_endpoint_flags(release, &did, &endpoint);
    }
}

fn commit_summary(repo: &surf::Repository, oid: radicle::git::Oid) -> Option<String> {
    let surf_oid = crate::oid::into_surf(oid);
    repo.commit(surf_oid).ok().map(|c| c.summary)
}
