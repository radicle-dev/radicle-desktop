use radicle::git;
use radicle::identity;

use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::release::{ReleaseFilter, Releases};

use crate::AppState;

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
