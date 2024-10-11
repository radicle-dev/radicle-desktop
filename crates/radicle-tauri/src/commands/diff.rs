use radicle::storage::ReadStorage;
use radicle_surf as surf;

use radicle::git;
use radicle::identity;
use serde::Deserialize;
use serde::Serialize;

use crate::{error, AppState};

#[derive(Serialize, Deserialize)]
pub struct Options {
    pub base: git::Oid,
    pub head: git::Oid,
    pub unified: u32,
}

#[tauri::command]
pub async fn get_diff(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    options: Options,
) -> Result<surf::diff::Diff, error::Error> {
    let repo = ctx.profile.storage.repository(rid)?.backend;
    let base = repo.find_commit(*options.base)?;
    let head = repo.find_commit(*options.head)?;

    let mut opts = git::raw::DiffOptions::new();
    opts.patience(true)
        .minimal(true)
        .context_lines(options.unified);

    let mut find_opts = git::raw::DiffFindOptions::new();
    find_opts.exact_match_only(true);
    find_opts.all(true);

    let left = base.tree()?;
    let right = head.tree()?;

    let mut diff = repo.diff_tree_to_tree(Some(&left), Some(&right), Some(&mut opts))?;
    diff.find_similar(Some(&mut find_opts))?;
    let diff = surf::diff::Diff::try_from(diff)?;

    Ok::<_, error::Error>(diff)
}
