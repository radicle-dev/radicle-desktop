use radicle::git;
use radicle::identity::RepoId;
use radicle::node::NodeId;
use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::repo::{Repo, Show};

use crate::AppState;

#[tauri::command]
pub async fn list_repos(
    ctx: tauri::State<'_, AppState>,
    show: Show,
) -> Result<Vec<types::repo::RepoInfo>, Error> {
    ctx.list_repos(show)
}

#[tauri::command]
pub async fn list_repos_summary(
    ctx: tauri::State<'_, AppState>,
) -> Result<Vec<types::repo::RepoSummary>, Error> {
    ctx.list_repos_summary()
}

#[tauri::command]
pub async fn repo_count(ctx: tauri::State<'_, AppState>) -> Result<types::repo::RepoCount, Error> {
    ctx.repo_count()
}

#[tauri::command]
pub async fn list_repo_refs(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
) -> Result<types::repo::RepoRefs, Error> {
    ctx.list_repo_refs(rid)
}

#[tauri::command]
pub async fn repo_by_id(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
) -> Result<types::repo::RepoInfo, Error> {
    ctx.repo_by_id(rid)
}

#[tauri::command]
pub async fn repo_readme(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    sha: Option<git::Oid>,
    peer: Option<NodeId>,
    revision: Option<String>,
) -> Result<Option<types::repo::Readme>, Error> {
    ctx.repo_readme(rid, sha, peer, revision)
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
    ctx.repo_tree(rid, path, sha, peer, revision)
}

#[tauri::command]
pub async fn repo_blob(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    path: std::path::PathBuf,
    sha: Option<git::Oid>,
) -> Result<types::source::blob::Blob, Error> {
    ctx.repo_blob(rid, path, sha)
}

#[tauri::command]
pub async fn diff_stats(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: git::Oid,
    head: git::Oid,
) -> Result<types::diff::Stats, Error> {
    ctx.diff_stats(rid, base, head)
}

#[tauri::command]
pub async fn list_commits(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: String,
    head: String,
) -> Result<Vec<types::repo::Commit>, Error> {
    ctx.list_commits(rid, base, head)
}

#[tauri::command]
pub async fn get_commit_diff(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    sha: git::Oid,
    unified: Option<u32>,
    highlight: Option<bool>,
) -> Result<types::diff::Diff, Error> {
    ctx.get_commit_diff(rid, sha, unified, highlight)
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
    ctx.list_repo_commits(rid, head, peer, revision, skip, take)
}

#[tauri::command]
pub async fn repo_commit_count(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    head: git::Oid,
) -> Result<usize, Error> {
    ctx.repo_commit_count(rid, head)
}

#[tauri::command]
pub async fn repo_commit(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    sha: Option<git::Oid>,
    peer: Option<NodeId>,
    revision: Option<String>,
) -> Result<types::repo::Commit, Error> {
    ctx.repo_commit(rid, sha, peer, revision)
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
