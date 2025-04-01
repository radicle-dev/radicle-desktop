use radicle::identity;
use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::repo::Repo;

use crate::AppState;

#[tauri::command]
pub async fn get_diff(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    options: radicle_types::cobs::diff::DiffOptions,
) -> Result<types::diff::Diff, Error> {
    ctx.get_diff(rid, options)
}
