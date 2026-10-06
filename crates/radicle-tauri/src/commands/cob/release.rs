use radicle::git;
use radicle::identity;

use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::release::{ReleaseFilter, Releases};
use radicle_types::traits::release_mut::ReleasesMut;

use tauri_plugin_dialog::DialogExt;

use crate::AppState;
use crate::commands::blocking;

#[tauri::command]
pub(crate) async fn list_releases(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    filter: Option<ReleaseFilter>,
    skip: Option<usize>,
    // None: return all releases, `skip` is ignored.
    take: Option<usize>,
) -> Result<types::cobs::PaginatedQuery<Vec<types::cobs::release::Release>>, Error> {
    blocking(ctx, move |ctx| ctx.list_releases(rid, filter, skip, take)).await
}

#[tauri::command]
pub(crate) async fn release_by_id(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Option<types::cobs::release::Release>, Error> {
    blocking(ctx, move |ctx| ctx.release_by_id(rid, id)).await
}

#[tauri::command]
pub(crate) async fn release_counts(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
) -> Result<types::cobs::release::ReleaseCounts, Error> {
    blocking(ctx, move |ctx| ctx.release_counts(rid)).await
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
pub async fn attest_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
    path: std::path::PathBuf,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| {
        ctx.attest_artifact(rid, release_id, cid, path)
    })
    .await
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
pub async fn redact_artifact(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
    cid: String,
    reason: String,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| {
        ctx.redact_artifact(rid, release_id, cid, reason)
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

#[tauri::command]
pub async fn delete_release(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    release_id: String,
) -> Result<(), Error> {
    blocking(ctx, move |ctx| ctx.delete_release(rid, release_id)).await
}
