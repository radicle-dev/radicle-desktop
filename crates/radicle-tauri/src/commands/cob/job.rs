use radicle::git;
use radicle::identity::RepoId;
use radicle_types::cobs::job;
use radicle_types::error::Error;
use radicle_types::traits::job::Jobs;

use crate::AppState;

#[tauri::command]
pub fn list_jobs(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    sha: git::Oid,
) -> Result<Vec<job::Job>, Error> {
    ctx.list_jobs(rid, sha)
}
