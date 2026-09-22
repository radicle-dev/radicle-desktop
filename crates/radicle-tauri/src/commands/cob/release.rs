use radicle::git;
use radicle::identity;

use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::artifact_node::ArtifactNode;
use radicle_types::traits::release::{ReleaseFilter, Releases};
use radicle_types::traits::release_mut::ReleasesMut;

use tauri::Emitter;
use tauri_plugin_dialog::DialogExt;

use crate::AppState;
use crate::commands::blocking;

#[tauri::command]
pub(crate) fn list_releases(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    filter: Option<ReleaseFilter>,
    skip: Option<usize>,
    // None: return all releases, `skip` is ignored.
    take: Option<usize>,
) -> Result<types::cobs::PaginatedQuery<Vec<types::cobs::release::Release>>, Error> {
    ctx.list_releases(rid, filter, skip, take)
}

#[tauri::command]
pub(crate) fn release_by_id(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Option<types::cobs::release::Release>, Error> {
    ctx.release_by_id(rid, id)
}

#[tauri::command]
pub(crate) fn release_count(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
) -> Result<usize, Error> {
    ctx.release_count(rid)
}

#[tauri::command]
pub async fn compute_artifact_cid(
    ctx: tauri::State<'_, AppState>,
    path: std::path::PathBuf,
) -> Result<types::cobs::release::ArtifactDigest, Error> {
    blocking(ctx, move |ctx| ctx.compute_artifact_cid(path)).await
}

#[tauri::command]
pub async fn create_or_open_release(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    oid: git::Oid,
    tag: Option<git::Oid>,
) -> Result<String, Error> {
    blocking(ctx, move |ctx| ctx.create_or_open_release(rid, oid, tag)).await
}

#[tauri::command]
pub async fn register_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
    name: String,
    size_bytes: u64,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| {
        ctx.register_artifact(rid, release_id, cid, name, size_bytes)
    })
    .await
}

#[tauri::command]
pub async fn add_artifact_location(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
    url: String,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| ctx.add_location(rid, release_id, cid, url)).await
}

#[tauri::command]
pub async fn remove_artifact_location(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
    url: String,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| {
        ctx.remove_location(rid, release_id, cid, url)
    })
    .await
}

#[tauri::command]
pub async fn attest_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| ctx.attest_artifact(rid, release_id, cid)).await
}

#[tauri::command]
pub async fn set_artifact_metadata(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
    key: String,
    value: serde_json::Value,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| {
        ctx.set_artifact_metadata(rid, release_id, cid, key, value)
    })
    .await
}

#[tauri::command]
pub async fn remove_artifact_metadata(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
    key: String,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| {
        ctx.remove_artifact_metadata(rid, release_id, cid, key)
    })
    .await
}

#[tauri::command]
pub async fn redact_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
    reason: String,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| {
        ctx.redact_and_unseed_artifact(rid, release_id, cid, reason)
    })
    .await
}

/// Pick one or more files to attach to a release. Returns an empty list when
/// the user cancels. The blocking dialog runs off the main thread, which the
/// plugin requires.
#[tauri::command]
pub async fn pick_artifact_files(app: tauri::AppHandle) -> Result<Vec<std::path::PathBuf>, Error> {
    let paths = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .blocking_pick_files()
            .unwrap_or_default()
    })
    .await?;

    Ok(paths
        .into_iter()
        .filter_map(|path| path.into_path().ok())
        .collect())
}

/// Pick a single directory to attach as a collection artifact.
#[tauri::command]
pub async fn pick_artifact_directory(
    app: tauri::AppHandle,
) -> Result<Option<std::path::PathBuf>, Error> {
    let path =
        tauri::async_runtime::spawn_blocking(move || app.dialog().file().blocking_pick_folder())
            .await?;

    Ok(path.and_then(|path| path.into_path().ok()))
}

/// Whether the local artifact node is running. A node that is down is not an
/// error here: the UI shows setup guidance instead of stats.
#[tauri::command]
pub async fn artifact_node_running(ctx: tauri::State<'_, AppState>) -> Result<bool, Error> {
    blocking(ctx, |ctx| Ok(ctx.artifact_node_running())).await
}

/// Which artifact binaries are installed, so a node that is down can be told
/// apart from one that was never installed.
#[tauri::command]
pub fn artifact_binaries() -> radicle_types::binaries::ArtifactBinaries {
    radicle_types::binaries::artifact_binaries()
}

#[tauri::command]
pub async fn artifact_node_status(
    ctx: tauri::State<'_, AppState>,
) -> Result<types::artifact::ArtifactNodeStatus, Error> {
    blocking(ctx, |ctx| ctx.artifact_node_status()).await
}

/// Content ids the node seeds for this repository. The release view asks once
/// and matches its rows against the result.
#[tauri::command]
pub async fn seeded_artifacts(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
) -> Result<Vec<String>, Error> {
    blocking(ctx, move |ctx| ctx.seeded_artifacts(rid)).await
}

#[tauri::command]
pub async fn is_seeding_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    cid: String,
) -> Result<bool, Error> {
    blocking(ctx, move |ctx| ctx.is_seeding_artifact(rid, cid)).await
}

#[tauri::command]
pub async fn seed_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
    source_path: std::path::PathBuf,
) -> Result<String, Error> {
    blocking(ctx, move |ctx| {
        ctx.seed_artifact(rid, release_id, cid, source_path)
    })
    .await
}

#[tauri::command]
pub async fn unseed_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| ctx.unseed_artifact(rid, release_id, cid)).await
}

/// Download an artifact to `dest`, forwarding the node's progress frames to
/// the frontend as `artifact_progress` events keyed by content id.
#[tauri::command]
pub async fn download_artifact(
    app: tauri::AppHandle,
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
    dest: std::path::PathBuf,
    seed: bool,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| {
        ctx.download_artifact(rid, release_id, cid.clone(), &dest, seed, |progress| {
            let _ = app.emit(
                "artifact_progress",
                types::artifact::ArtifactProgress::new(&cid, progress),
            );
        })
    })
    .await
}

/// Open the OS save dialog seeded with `suggested_name`, returning the chosen
/// path or `None` when the user cancels.
#[tauri::command]
pub async fn pick_artifact_save_path(
    app: tauri::AppHandle,
    suggested_name: String,
) -> Result<Option<std::path::PathBuf>, Error> {
    let path = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_file_name(suggested_name)
            .blocking_save_file()
    })
    .await?;

    Ok(path.and_then(|path| path.into_path().ok()))
}
