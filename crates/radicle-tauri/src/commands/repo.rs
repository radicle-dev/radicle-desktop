use radicle::git;
use radicle::identity::RepoId;
use radicle::node::NodeId;
use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::repo::{Repo, Show};

use crate::AppState;
use crate::commands::blocking;

#[tauri::command]
pub async fn list_repos(
    ctx: tauri::State<'_, AppState>,
    show: Show,
) -> Result<Vec<types::repo::RepoInfo>, Error> {
    blocking(ctx, move |ctx| ctx.list_repos(show)).await
}

#[tauri::command]
pub async fn list_repos_summary(
    ctx: tauri::State<'_, AppState>,
) -> Result<Vec<types::repo::RepoSummary>, Error> {
    blocking(ctx, |ctx| ctx.list_repos_summary()).await
}

#[tauri::command]
pub async fn repo_count(ctx: tauri::State<'_, AppState>) -> Result<types::repo::RepoCount, Error> {
    blocking(ctx, |ctx| ctx.repo_count()).await
}

#[tauri::command]
pub async fn list_repo_refs(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
) -> Result<types::repo::RepoRefs, Error> {
    blocking(ctx, move |ctx| ctx.list_repo_refs(rid)).await
}

#[tauri::command]
pub async fn repo_by_id(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
) -> Result<types::repo::RepoInfo, Error> {
    blocking(ctx, move |ctx| ctx.repo_by_id(rid)).await
}

#[tauri::command]
pub async fn repo_readme(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    sha: Option<git::Oid>,
    peer: Option<NodeId>,
    revision: Option<String>,
) -> Result<Option<types::repo::Readme>, Error> {
    blocking(ctx, move |ctx| ctx.repo_readme(rid, sha, peer, revision)).await
}

#[tauri::command]
pub async fn repo_tree(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    path: std::path::PathBuf,
    sha: Option<git::Oid>,
    peer: Option<NodeId>,
    revision: Option<String>,
) -> Result<types::source::tree::Tree, Error> {
    blocking(ctx, move |ctx| {
        ctx.repo_tree(rid, path, sha, peer, revision)
    })
    .await
}

#[tauri::command]
pub async fn repo_blob(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    path: std::path::PathBuf,
    sha: Option<git::Oid>,
) -> Result<types::source::blob::Blob, Error> {
    blocking(ctx, move |ctx| ctx.repo_blob(rid, path, sha)).await
}

#[tauri::command]
pub async fn diff_stats(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: git::Oid,
    head: git::Oid,
) -> Result<types::diff::Stats, Error> {
    blocking(ctx, move |ctx| ctx.diff_stats(rid, base, head)).await
}

#[tauri::command]
pub async fn list_commits(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: String,
    head: String,
) -> Result<Vec<types::repo::Commit>, Error> {
    blocking(ctx, move |ctx| ctx.list_commits(rid, base, head)).await
}

#[tauri::command]
pub async fn get_commit_diff(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    sha: git::Oid,
    unified: Option<u32>,
    highlight: Option<bool>,
) -> Result<types::diff::Diff, Error> {
    blocking(ctx, move |ctx| {
        ctx.get_commit_diff(rid, sha, unified, highlight)
    })
    .await
}

#[tauri::command]
pub async fn list_repo_commits(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    head: Option<git::Oid>,
    peer: Option<NodeId>,
    revision: Option<String>,
    skip: Option<usize>,
    take: Option<usize>,
) -> Result<types::cobs::PaginatedQuery<Vec<types::repo::Commit>>, Error> {
    blocking(ctx, move |ctx| {
        ctx.list_repo_commits(rid, head, peer, revision, skip, take)
    })
    .await
}

#[tauri::command]
pub async fn repo_commit_count(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    head: git::Oid,
) -> Result<usize, Error> {
    blocking(ctx, move |ctx| ctx.repo_commit_count(rid, head)).await
}

#[tauri::command]
pub async fn repo_commit(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    sha: Option<git::Oid>,
    peer: Option<NodeId>,
    revision: Option<String>,
) -> Result<types::repo::Commit, Error> {
    blocking(ctx, move |ctx| ctx.repo_commit(rid, sha, peer, revision)).await
}

#[tauri::command]
pub async fn list_tags(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
) -> Result<Vec<types::repo::TagRef>, Error> {
    let ctx = ctx.inner().clone();
    tauri::async_runtime::spawn_blocking(move || ctx.list_tags(rid)).await?
}

#[tauri::command]
pub fn seed(ctx: tauri::State<'_, AppState>, rid: RepoId) -> Result<(), Error> {
    ctx.seed(rid)
}

#[tauri::command]
pub fn unseed(ctx: tauri::State<'_, AppState>, rid: RepoId) -> Result<(), Error> {
    ctx.unseed(rid)
}

#[tauri::command]
pub async fn seeded_not_replicated(ctx: tauri::State<'_, AppState>) -> Result<Vec<RepoId>, Error> {
    ctx.seeded_not_replicated()
}
