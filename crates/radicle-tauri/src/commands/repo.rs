use radicle::git;
use radicle::identity::RepoId;

use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::repo::{Repo, Show};

use crate::AppState;

#[tauri::command]
pub fn list_repos(
    ctx: tauri::State<AppState>,
    show: Show,
) -> Result<Vec<types::repo::RepoInfo>, Error> {
    ctx.list_repos(show)
}

#[tauri::command]
pub fn repo_by_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
) -> Result<types::repo::RepoInfo, Error> {
    ctx.repo_by_id(rid)
}

#[tauri::command]
pub async fn diff_stats(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: git::Oid,
    head: git::Oid,
) -> Result<types::cobs::Stats, Error> {
    ctx.diff_stats(rid, base, head)
}

#[tauri::command]
pub async fn list_commits(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    parent: Option<String>,
    skip: Option<usize>,
    take: Option<usize>,
) -> Result<types::cobs::PaginatedQuery<Vec<types::repo::Commit>>, Error> {
    ctx.list_commits(rid, parent, skip, take)
}
