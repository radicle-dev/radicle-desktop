use radicle_surf as surf;

use radicle::identity;
use radicle_types::traits::repo::Repo;

use crate::{error, AppState};

#[tauri::command]
pub async fn get_diff(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    options: radicle_types::cobs::diff::Options,
) -> Result<surf::diff::Diff, error::Error> {
    ctx.get_diff(rid, options).map_err(error::Error::from)
}
