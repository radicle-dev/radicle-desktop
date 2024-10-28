use radicle::git;
use radicle::identity::RepoId;

use radicle_types as types;
use radicle_types::traits::repo::Repo;

use crate::error::Error;
use crate::AppState;

#[tauri::command]
pub fn list_repos(ctx: tauri::State<AppState>) -> Result<Vec<types::repo::RepoInfo>, Error> {
    ctx.list_repos().map_err(Error::from)
}

#[tauri::command]
pub fn repo_by_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
) -> Result<types::repo::RepoInfo, Error> {
    ctx.repo_by_id(rid).map_err(Error::from)
}

#[tauri::command]
pub async fn diff_stats(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: git::Oid,
    head: git::Oid,
) -> Result<types::cobs::Stats, Error> {
    ctx.diff_stats(rid, base, head).map_err(Error::from)
}
