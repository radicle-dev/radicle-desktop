use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use cid::Cid;
use radicle::git;
use radicle::identity::{Did, RepoId};
use radicle_artifact_client::DownloadArgs;
use radicle_artifact_core::cid as cid_utils;
use radicle_artifact_core::keys::EndpointId;
use radicle_artifact_core::protocol::{FetchLocation, FetchProgress, ImportMode};
use url::Url;

use radicle_types::ArtifactClient;
use radicle_types::artifact::ArtifactNodeStatus;
use radicle_types::cobs::release;
use radicle_types::error::Error;
use radicle_types::traits::release::Releases;
use radicle_types::traits::release_mut::ReleasesMut;

use crate::AppState;

/// Per-frame idle timeout for streaming fetch calls to the node. Bounds
/// the wait for each progress frame, not the whole transfer.
const FETCH_IDLE: Duration = Duration::from_secs(30);

#[tauri::command]
pub async fn list_releases(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
) -> Result<Vec<release::Release>, Error> {
    let ctx = ctx.inner().clone();
    let releases = tauri::async_runtime::spawn_blocking(move || ctx.list_releases(rid)).await??;
    Ok(releases)
}

#[tauri::command]
pub async fn release_by_id(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    release_id: String,
) -> Result<Option<release::Release>, Error> {
    let ctx = ctx.inner().clone();
    let release =
        tauri::async_runtime::spawn_blocking(move || ctx.release_by_id(rid, release_id)).await??;
    Ok(release)
}

#[tauri::command]
pub async fn releases_by_commit(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    sha: git::Oid,
) -> Result<Vec<release::Release>, Error> {
    let ctx = ctx.inner().clone();
    let releases =
        tauri::async_runtime::spawn_blocking(move || ctx.releases_by_commit(rid, sha)).await??;
    Ok(releases)
}

/// CID computation can be expensive on large files; off-load to the
/// blocking pool so the IPC thread stays responsive.
#[tauri::command]
pub async fn compute_artifact_cid(
    ctx: tauri::State<'_, AppState>,
    path: PathBuf,
) -> Result<String, Error> {
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.compute_cid(path)).await?
}

#[tauri::command]
pub async fn create_or_open_release(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    oid: git::Oid,
    tag: Option<git::Oid>,
) -> Result<String, Error> {
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.create_or_open_release(rid, oid, tag)).await?
}

#[tauri::command]
pub async fn register_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    release_id: String,
    cid: String,
    name: String,
) -> Result<(), Error> {
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.register_artifact(rid, release_id, cid, name))
        .await?
}

#[tauri::command]
pub async fn add_location(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    release_id: String,
    cid: String,
    url: String,
) -> Result<(), Error> {
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.add_location(rid, release_id, cid, url))
        .await?
}

#[tauri::command]
pub async fn remove_location(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    release_id: String,
    cid: String,
    url: String,
) -> Result<(), Error> {
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.remove_location(rid, release_id, cid, url))
        .await?
}

#[tauri::command]
pub async fn attest_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    release_id: String,
    cid: String,
) -> Result<(), Error> {
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.attest_artifact(rid, release_id, cid)).await?
}

#[tauri::command]
pub async fn set_metadata(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    release_id: String,
    cid: String,
    key: String,
    value: serde_json::Value,
) -> Result<(), Error> {
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.set_metadata(rid, release_id, cid, key, value))
        .await?
}

#[tauri::command]
pub async fn remove_metadata(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    release_id: String,
    cid: String,
    key: String,
) -> Result<(), Error> {
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.remove_metadata(rid, release_id, cid, key))
        .await?
}

#[tauri::command]
pub async fn redact_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    release_id: String,
    cid: String,
    reason: String,
) -> Result<(), Error> {
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.redact_artifact(rid, release_id, cid, reason))
        .await?
}

// Seed / unseed -------------------------------------------------------------

#[tauri::command]
pub async fn seed_artifact(
    ctx: tauri::State<'_, AppState>,
    artifact: tauri::State<'_, ArtifactClient>,
    rid: RepoId,
    release_id: String,
    cid: String,
    source_path: PathBuf,
) -> Result<String, Error> {
    let parsed_cid = Cid::from_str(&cid)?;
    let kind = cid_utils::artifact_kind(&parsed_cid)?;

    // Ask the node to import the bytes (ImportMode::Copy so the source can
    // be moved or deleted later) and tag the (rid, cid) as seeded.
    let receipt = artifact
        .client
        .seed(rid, parsed_cid, &source_path, kind, ImportMode::Copy)
        .await?;

    // Announce the node's endpoint on the COB so peers can discover us.
    let url = receipt.endpoint_id.to_url().to_string();
    let url_for_return = url.clone();
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.add_location(rid, release_id, cid, url))
        .await??;

    Ok(url_for_return)
}

#[tauri::command]
pub async fn unseed_artifact(
    ctx: tauri::State<'_, AppState>,
    artifact: tauri::State<'_, ArtifactClient>,
    rid: RepoId,
    release_id: String,
    cid: String,
) -> Result<(), Error> {
    let parsed_cid = Cid::from_str(&cid)?;

    // Drop the COB location first so peers stop trying to reach us before
    // we untag the blob. The URL we announced is our DID-derived endpoint.
    let our_did = Did::from(ctx.profile.public_key);
    let url = EndpointId::try_from(&our_did)?.to_url().to_string();
    let cid_for_remove = cid.clone();
    let ctx_clone = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        ctx_clone.remove_location(rid, release_id, cid_for_remove, url)
    })
    .await??;

    artifact.client.unseed(rid, parsed_cid).await?;
    Ok(())
}

#[tauri::command]
pub async fn is_seeding(
    artifact: tauri::State<'_, ArtifactClient>,
    rid: RepoId,
    cid: String,
) -> Result<bool, Error> {
    let parsed_cid = Cid::from_str(&cid)?;
    Ok(artifact.client.is_seeding(rid, parsed_cid).await?)
}

/// Current status of the rad-artifact node — endpoint, uptime, seeding
/// totals, connection and traffic counters. Errors with
/// `ArtifactNodeNotRunning` when the node is down so the UI can show setup
/// guidance instead of stats.
#[tauri::command]
pub async fn artifact_node_status(
    artifact: tauri::State<'_, ArtifactClient>,
) -> Result<ArtifactNodeStatus, Error> {
    let status = artifact.client.status().await?;
    Ok(ArtifactNodeStatus::from(status))
}

// Download ------------------------------------------------------------------

use tauri::Emitter;

/// Resolve COB locations into the concrete `FetchLocation` list the node
/// fetches from. Endpoint (`radiroh://`) URLs become iroh locations — a
/// bare host derives from the contributing DID; every other scheme is
/// passed through as a URL. Legacy `iroh://` locations are ignored.
fn resolve_fetch_locations(locations: &BTreeMap<Did, BTreeSet<Url>>) -> Vec<FetchLocation> {
    let mut seen_ids: HashSet<EndpointId> = HashSet::new();
    let mut seen_urls: HashSet<&Url> = HashSet::new();
    let mut out = Vec::new();
    for (did, set) in locations {
        for url in set {
            if EndpointId::is_endpoint_url(url) {
                let id = match EndpointId::from_url(url) {
                    Ok(Some(id)) => Some(id),
                    Ok(None) => EndpointId::try_from(did).ok(),
                    Err(_) => None,
                };
                if let Some(id) = id
                    && seen_ids.insert(id)
                {
                    out.push(FetchLocation::Iroh(id));
                }
            } else if EndpointId::is_legacy_endpoint_url(url) {
                // Stale pre-rename location; ignore on read.
                continue;
            } else if seen_urls.insert(url) {
                out.push(FetchLocation::Url(url.clone()));
            }
        }
    }
    out
}

/// Fetch an artifact from its COB locations and write it to `dest`. The
/// node owns every transport (local fast-path, HTTP, iroh) and the export
/// to disk; we resolve locations, stream the node's progress frames into
/// the `artifact_progress` event keyed by `cid`, and announce ourselves
/// when auto-seed is on (the node tags it as seeded during the fetch).
#[tauri::command]
pub async fn download_artifact(
    app: tauri::AppHandle,
    ctx: tauri::State<'_, AppState>,
    artifact: tauri::State<'_, ArtifactClient>,
    rid: RepoId,
    release_id: String,
    cid: String,
    dest: PathBuf,
) -> Result<(), Error> {
    let parsed_cid = Cid::from_str(&cid)?;

    // Snapshot the (did, urls) locations synchronously so we hold no COB
    // lock during the fetch.
    let cid_for_locations = cid.clone();
    let release_id_for_locations = release_id.clone();
    let ctx_clone = ctx.inner().clone();
    let locations = tauri::async_runtime::spawn_blocking(move || {
        ctx_clone.artifact_locations(rid, release_id_for_locations, cid_for_locations)
    })
    .await??;
    let fetch_locations = resolve_fetch_locations(&locations);

    let auto_seed = radicle_types::settings::load(ctx.profile.home().path()).auto_seed_artifacts;

    // Bridge the node's streamed progress frames to the per-CID UI event.
    let cid_for_progress = cid.clone();
    let app_for_progress = app.clone();
    let on_progress = move |p: &FetchProgress| {
        let payload = match p {
            FetchProgress::Connecting
            | FetchProgress::TryingLocation { .. }
            | FetchProgress::LocationFailed { .. } => {
                serde_json::json!({ "cid": cid_for_progress, "stage": "connecting" })
            }
            FetchProgress::Downloading { offset, .. } => serde_json::json!({
                "cid": cid_for_progress,
                "stage": "downloading",
                "bytes": offset,
            }),
            FetchProgress::Exporting { offset, .. } => serde_json::json!({
                "cid": cid_for_progress,
                "stage": "writing",
                "bytes": offset,
            }),
        };
        let _ = app_for_progress.emit("artifact_progress", payload);
    };

    let receipt = artifact
        .client
        .download(
            DownloadArgs {
                rid,
                cid: parsed_cid,
                locations: fetch_locations,
                dest,
                seed: auto_seed,
            },
            FETCH_IDLE,
            on_progress,
        )
        .await?;

    // The node tagged it as seeded; announce our endpoint on the COB so
    // peers can discover the new mirror.
    if receipt.seeded {
        let url = receipt.endpoint_id.to_url().to_string();
        let cid_for_location = cid.clone();
        let ctx_clone = ctx.inner().clone();
        tauri::async_runtime::spawn_blocking(move || {
            ctx_clone.add_location(rid, release_id, cid_for_location, url)
        })
        .await??;
    }

    let _ = app.emit(
        "artifact_progress",
        serde_json::json!({ "cid": cid, "stage": "done" }),
    );
    Ok(())
}

// File picker ---------------------------------------------------------------

use tauri_plugin_dialog::DialogExt;

/// Open a multi-file picker. Returns the selected paths as strings, or an
/// empty Vec if the user cancelled. The frontend feeds each path into
/// `compute_artifact_cid` then `register_artifact`.
#[tauri::command]
pub async fn pick_artifact_files(app: tauri::AppHandle) -> Result<Vec<String>, Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_files(move |paths| {
        let _ = tx.send(paths.unwrap_or_default());
    });
    let paths = rx.await.map_err(|_| Error::DialogClosed)?;
    Ok(paths
        .into_iter()
        .filter_map(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

/// Open a single-directory picker. Used when the user wants to attach a
/// directory artifact (which becomes a Collection CID).
#[tauri::command]
pub async fn pick_artifact_directory(app: tauri::AppHandle) -> Result<Option<String>, Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path);
    });
    let path = rx.await.map_err(|_| Error::DialogClosed)?;
    Ok(path
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned()))
}

/// Open the OS "Save as" dialog seeded with `suggested_name`. Returns the
/// path the user picked, or `None` if they cancelled. The frontend feeds
/// the path into `download_artifact` as the destination for a Blob fetch.
#[tauri::command]
pub async fn pick_artifact_save_path(
    app: tauri::AppHandle,
    suggested_name: String,
) -> Result<Option<String>, Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_file_name(suggested_name)
        .save_file(move |path| {
            let _ = tx.send(path);
        });
    let path = rx.await.map_err(|_| Error::DialogClosed)?;
    Ok(path
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned()))
}

// Settings ------------------------------------------------------------------

#[tauri::command]
pub fn get_auto_seed_artifacts(ctx: tauri::State<AppState>) -> Result<bool, Error> {
    let settings = radicle_types::settings::load(ctx.profile.home().path());
    Ok(settings.auto_seed_artifacts)
}

#[tauri::command]
pub fn set_auto_seed_artifacts(ctx: tauri::State<AppState>, enabled: bool) -> Result<(), Error> {
    let mut settings = radicle_types::settings::load(ctx.profile.home().path());
    settings.auto_seed_artifacts = enabled;
    radicle_types::settings::save(ctx.profile.home().path(), &settings)
}
