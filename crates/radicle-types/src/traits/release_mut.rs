use std::str::FromStr;

use radicle::identity;
use radicle::storage::ReadStorage;
use radicle_artifact::Releases as ReleasesStore;
use url::Url;

use crate::error::Error;
use crate::traits::release::Releases;

pub trait ReleasesMut: Releases {
    /// Find a release for the commit OID, or create it. Returns the
    /// release id as a string for the frontend. Idempotent.
    ///
    /// `tag` is recorded on the release COB when the user selects an
    /// annotated tag for the release; for lightweight tags or bare
    /// commit OIDs pass `None`. Only honoured when creating a release —
    /// when an existing one is reused, its tag stays as-is.
    ///
    /// radicle-artifact 0.12 dropped its built-in `find_or_create_by_oid`
    /// in favour of an explicit two-step pattern; we recreate that here.
    fn create_or_open_release(
        &self,
        rid: identity::RepoId,
        oid: radicle::git::Oid,
        tag: Option<radicle::git::Oid>,
    ) -> Result<String, Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let mut releases = ReleasesStore::open(&repo)?;

        let existing = {
            let mut iter = releases.find_by_commit(oid)?;
            match iter.next() {
                Some(item) => Some(item?.0),
                None => None,
            }
        };
        let id = match existing {
            Some(id) => id,
            None => {
                let release = releases.create(oid, tag, &signer)?;
                *release.id()
            }
        };
        Ok(id.to_string())
    }

    fn register_artifact(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
        name: String,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let id = radicle_artifact::ReleaseId::from_str(&release_id)?;
        let cid = radicle_artifact::Cid::from_str(&cid)?;

        let mut releases = ReleasesStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        // 0.14 renamed `register_artifact` to `register_artifact`; same args.
        release.register_artifact(cid, name, &signer)?;
        Ok(())
    }

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

        let id = radicle_artifact::ReleaseId::from_str(&release_id)?;
        let cid = radicle_artifact::Cid::from_str(&cid)?;
        let url = Url::parse(&url)?;

        let mut releases = ReleasesStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.add_location(cid, url, &signer)?;
        Ok(())
    }

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

        let id = radicle_artifact::ReleaseId::from_str(&release_id)?;
        let cid = radicle_artifact::Cid::from_str(&cid)?;
        let url = Url::parse(&url)?;

        let mut releases = ReleasesStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.remove_location(cid, url, &signer)?;
        Ok(())
    }

    fn attest_artifact(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let id = radicle_artifact::ReleaseId::from_str(&release_id)?;
        let cid = radicle_artifact::Cid::from_str(&cid)?;

        let mut releases = ReleasesStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.attest(cid, &signer)?;
        Ok(())
    }

    fn set_metadata(
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

        let id = radicle_artifact::ReleaseId::from_str(&release_id)?;
        let cid = radicle_artifact::Cid::from_str(&cid)?;

        let mut releases = ReleasesStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.set_metadata(cid, key, value, &signer)?;
        Ok(())
    }

    fn remove_metadata(
        &self,
        rid: identity::RepoId,
        release_id: String,
        cid: String,
        key: String,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;

        let id = radicle_artifact::ReleaseId::from_str(&release_id)?;
        let cid = radicle_artifact::Cid::from_str(&cid)?;

        let mut releases = ReleasesStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.remove_metadata(cid, key, &signer)?;
        Ok(())
    }

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

        let id = radicle_artifact::ReleaseId::from_str(&release_id)?;
        let cid = radicle_artifact::Cid::from_str(&cid)?;

        let mut releases = ReleasesStore::open(&repo)?;
        let mut release = releases.get_mut(&id)?;
        release.redact(cid, reason, &signer)?;
        Ok(())
    }
}
