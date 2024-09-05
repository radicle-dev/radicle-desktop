use radicle::storage::ReadStorage;

use crate::error::Error;
use crate::types;
use crate::AppState;

/// List all repos.
#[tauri::command]
pub fn list_repos(ctx: tauri::State<AppState>) -> Result<Vec<types::repo::RepoInfo>, Error> {
    let storage = &ctx.profile.storage;
    let policies = ctx.profile.policies()?;

    let mut repos = storage.repositories()?.into_iter().collect::<Vec<_>>();
    repos.sort_by_key(|p| p.rid);

    let infos = repos
        .into_iter()
        .filter_map(|info| {
            if !policies.is_seeding(&info.rid).unwrap_or_default() {
                return None;
            }
            let (repo, doc) = ctx.repo(info.rid).ok()?;
            let repo_info = ctx.repo_info(&repo, doc).ok()?;

            Some(repo_info)
        })
        .collect::<Vec<_>>();

    Ok::<_, Error>(infos)
}
