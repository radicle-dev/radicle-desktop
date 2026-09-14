use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use radicle::identity::{Did, RepoId};
use radicle::storage::ReadStorage;
use url::Url;

use radicle_artifact::{Cid, ReleaseId, Releases as ArtifactStore, cache_db_path};
use radicle_artifact_client::sync::Client;
use radicle_artifact_client::{DownloadArgs, default_socket};
use radicle_artifact_core::cid as cid_utils;
use radicle_artifact_core::keys::EndpointId;
use radicle_artifact_core::protocol::{FetchLocation, FetchProgress, ImportMode};

use crate::artifact::ArtifactNodeStatus;
use crate::error::Error;
use crate::traits::release_mut::ReleasesMut;

/// Per-frame idle timeout for streaming calls. This bounds the wait for each
/// progress frame, not the whole transfer, so a download that keeps making
/// progress never times out while a stalled one does.
const FETCH_IDLE: Duration = Duration::from_secs(30);

/// Resolve the locations recorded on a COB into the concrete list the node
/// fetches from.
///
/// Endpoint (`radiroh://`) URLs become iroh locations; a bare host falls back
/// to the contributing DID's own endpoint. Stale pre-rename `iroh://`
/// locations are dropped, and every other scheme is passed through as a URL.
fn resolve_fetch_locations(locations: &BTreeMap<Did, BTreeSet<Url>>) -> Vec<FetchLocation> {
    let mut seen_ids: HashSet<EndpointId> = HashSet::new();
    let mut seen_urls: HashSet<&Url> = HashSet::new();
    let mut resolved = Vec::new();

    for (did, urls) in locations {
        for url in urls {
            if EndpointId::is_endpoint_url(url) {
                let id = match EndpointId::from_url(url) {
                    Ok(Some(id)) => Some(id),
                    Ok(None) => EndpointId::try_from(did).ok(),
                    Err(_) => None,
                };
                if let Some(id) = id
                    && seen_ids.insert(id)
                {
                    resolved.push(FetchLocation::Iroh(id));
                }
            } else if EndpointId::is_legacy_endpoint_url(url) {
                continue;
            } else if seen_urls.insert(url) {
                resolved.push(FetchLocation::Url(url.clone()));
            }
        }
    }

    resolved
}

pub trait ArtifactNode: ReleasesMut {
    /// Client for the local artifact node's control socket.
    fn artifact_client(&self) -> Client {
        Client::new(default_socket(self.profile().home.path()))
    }

    /// Whether the artifact node is up. Used to decide between showing node
    /// stats and showing setup guidance, so a down node is not an error here.
    fn artifact_node_running(&self) -> bool {
        self.artifact_client().is_running()
    }

    /// Status of the local artifact node.
    fn artifact_node_status(&self) -> Result<ArtifactNodeStatus, Error> {
        Ok(ArtifactNodeStatus::from(self.artifact_client().status()?))
    }

    /// Whether the node currently seeds this artifact for the repository.
    fn is_seeding_artifact(&self, rid: RepoId, cid: String) -> Result<bool, Error> {
        let cid = Cid::from_str(&cid)
            .map_err(|err| radicle_artifact_core::Error::Cid(err.to_string()))?;

        Ok(self.artifact_client().is_seeding(rid, cid)?)
    }

    /// The locations recorded on the COB for one artifact, keyed by the node
    /// that contributed them.
    fn artifact_locations(
        &self,
        rid: RepoId,
        release_id: String,
        cid: String,
    ) -> Result<BTreeMap<Did, BTreeSet<Url>>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let id = ReleaseId::from_str(&release_id)?;
        let cid = Cid::from_str(&cid)
            .map_err(|err| radicle_artifact_core::Error::Cid(err.to_string()))?;

        let store = ArtifactStore::open_cached(&repo, cache_db_path(profile.cobs()))?;
        let Some(release) = store.get(&id)? else {
            return Ok(BTreeMap::new());
        };

        Ok(release
            .artifacts()
            .get(&cid)
            .map(|artifact| artifact.locations().clone())
            .unwrap_or_default())
    }

    /// Import a local file into the node and seed it, then announce the node's
    /// endpoint on the COB so peers can discover it. Returns the announced URL.
    fn seed_artifact(
        &self,
        rid: RepoId,
        release_id: String,
        cid: String,
        source_path: PathBuf,
    ) -> Result<String, Error> {
        let parsed = Cid::from_str(&cid)
            .map_err(|err| radicle_artifact_core::Error::Cid(err.to_string()))?;
        let kind = cid_utils::artifact_kind(&parsed)?;
        let release = *radicle::cob::ObjectId::from_str(&release_id)?;

        // `ImportMode::Copy` so the user can move or delete the source
        // afterwards without breaking what the node serves.
        let receipt = self.artifact_client().seed(
            rid,
            release,
            parsed,
            source_path.as_path(),
            kind,
            ImportMode::Copy,
        )?;

        let url = receipt.endpoint_id.to_url().to_string();
        self.add_location(rid, release_id, cid, url.clone())?;

        Ok(url)
    }

    /// Stop seeding an artifact and withdraw the location this node announced.
    fn unseed_artifact(&self, rid: RepoId, release_id: String, cid: String) -> Result<(), Error> {
        let parsed = Cid::from_str(&cid)
            .map_err(|err| radicle_artifact_core::Error::Cid(err.to_string()))?;
        let release = *radicle::cob::ObjectId::from_str(&release_id)?;

        // Drop the COB location first, so peers stop trying to reach us before
        // the bytes stop being served.
        let our_did = Did::from(self.profile().public_key);
        let url = EndpointId::try_from(&our_did)
            .map_err(|err| radicle_artifact_core::Error::Cid(err.to_string()))?
            .to_url()
            .to_string();
        self.remove_location(rid, release_id, cid, url)?;

        // Untag only this release's seed, so other releases sharing the CID
        // keep theirs.
        self.artifact_client().unseed(rid, Some(release), parsed)?;

        Ok(())
    }

    /// Fetch an artifact from the locations on its COB and write it to `dest`.
    ///
    /// The node owns every transport and the export to disk; this resolves the
    /// locations and forwards the node's progress frames to `on_progress`.
    /// With `seed` set, the node tags the bytes under the release as it goes.
    fn download_artifact(
        &self,
        rid: RepoId,
        release_id: String,
        cid: String,
        dest: &Path,
        seed: bool,
        on_progress: impl FnMut(&FetchProgress),
    ) -> Result<(), Error> {
        let parsed = Cid::from_str(&cid)
            .map_err(|err| radicle_artifact_core::Error::Cid(err.to_string()))?;
        let seed = seed
            .then(|| radicle::cob::ObjectId::from_str(&release_id))
            .transpose()?
            .map(|id| *id);

        let locations = self.artifact_locations(rid, release_id, cid)?;

        self.artifact_client().download(
            DownloadArgs {
                rid,
                cid: parsed,
                locations: resolve_fetch_locations(&locations),
                dest: dest.to_path_buf(),
                seed,
            },
            FETCH_IDLE,
            on_progress,
        )?;

        Ok(())
    }
}
