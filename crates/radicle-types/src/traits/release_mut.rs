use std::path::PathBuf;
use std::str::FromStr;

use radicle::identity;
use radicle::storage::ReadStorage;
use url::Url;

use radicle_artifact::{Cid, ReleaseId, Releases as ArtifactStore};
use radicle_artifact_core::cid as cid_utils;

use crate::cobs;
use crate::error::Error;
use crate::traits::release::Releases;

/// Parse a content id, folding the multiformats parse error into our own so
/// the rest of the app need not depend on the `cid` crate.
fn parse_cid(cid: &str) -> Result<Cid, Error> {
    Cid::from_str(cid).map_err(|err| radicle_artifact_core::Error::Cid(err.to_string()).into())
}

pub trait ReleasesMut: Releases {
    /// Content-address a file or directory on disk without involving the
    /// artifact node, so a release can be assembled while the node is down.
    ///
    /// Files hash as a single blob; directories hash as a collection over a
    /// canonical walk. The size is returned alongside so the caller can
    /// register the artifact and its size hint in one signed entry.
    fn compute_artifact_cid(&self, path: PathBuf) -> Result<cobs::release::ArtifactDigest, Error> {
        let directory = path.is_dir();
        let cid = if directory {
            cid_utils::compute_content_id(&path)?
        } else {
            cid_utils::compute_blob_cid(&path)?
        };
        let file_count = if directory {
            cid_utils::canonical_walk(&path)?.len() as u64
        } else {
            1
        };

        Ok(cobs::release::ArtifactDigest {
            cid: cid.to_string(),
            size_bytes: cid_utils::compute_size_from_path(&path)?,
            file_count,
            directory,
        })
    }

    /// Find the release for a commit, or create one. Returns the release id.
    ///
    /// Idempotent: re-running against the same commit reuses the existing
    /// release rather than creating a second one. `tag` is recorded only when
    /// the release is created; reusing an existing release leaves its tag
    /// as-is. Pass `None` for lightweight tags and bare commits, since the
    /// store accepts only an annotated tag that peels to `oid`.
    fn create_or_open_release(
        &self,
        rid: identity::RepoId,
        oid: radicle::git::Oid,
        tag: Option<radicle::git::Oid>,
    ) -> Result<String, Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let mut releases = ArtifactStore::open(&repo)?;

        let existing = releases
            .find_by_commit(oid)?
            .next()
            .transpose()?
            .map(|e| e.0);

        let id = match existing {
            Some(id) => id,
            None => *releases.create(oid, tag, &signer)?.id(),
        };

        Ok(id.to_string())
    }

    /// Register an artifact under a release, recording its size hint in the
    /// same signed entry.
    fn register_artifact(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
        name: String,
        size_bytes: u64,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let id = ReleaseId::from_str(&release_id)?;
        let cid = parse_cid(&cid)?;

        let mut releases = ArtifactStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.register_artifact_with_size(cid, name, size_bytes, &signer)?;

        Ok(())
    }

    /// Announce a place an artifact can be fetched from.
    fn add_location(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
        url: String,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let id = ReleaseId::from_str(&release_id)?;
        let cid = parse_cid(&cid)?;
        let url = Url::parse(&url)?;

        let mut releases = ArtifactStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.add_location(cid, url, &signer)?;

        Ok(())
    }

    /// Withdraw a location this node previously announced.
    fn remove_location(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
        url: String,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let id = ReleaseId::from_str(&release_id)?;
        let cid = parse_cid(&cid)?;
        let url = Url::parse(&url)?;

        let mut releases = ArtifactStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.remove_location(cid, url, &signer)?;

        Ok(())
    }

    /// Vouch that the artifact's bytes match its content id.
    fn attest_artifact(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let id = ReleaseId::from_str(&release_id)?;
        let cid = parse_cid(&cid)?;

        let mut releases = ArtifactStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.attest(cid, &signer)?;

        Ok(())
    }

    /// Set a free-form metadata key on an artifact.
    fn set_artifact_metadata(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let id = ReleaseId::from_str(&release_id)?;
        let cid = parse_cid(&cid)?;

        let mut releases = ArtifactStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.set_metadata(cid, key, value, &signer)?;

        Ok(())
    }

    /// Remove a metadata key from an artifact.
    fn remove_artifact_metadata(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
        key: String,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let id = ReleaseId::from_str(&release_id)?;
        let cid = parse_cid(&cid)?;

        let mut releases = ArtifactStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.remove_metadata(cid, key, &signer)?;

        Ok(())
    }

    /// Flag an artifact as one that should no longer be fetched.
    fn redact_artifact(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
        reason: String,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let id = ReleaseId::from_str(&release_id)?;
        let cid = parse_cid(&cid)?;

        let mut releases = ArtifactStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.redact(cid, reason, &signer)?;

        Ok(())
    }
}
