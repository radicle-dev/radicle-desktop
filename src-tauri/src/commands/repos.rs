use radicle::identity::RepoId;
use radicle::storage::ReadRepository;
use radicle::storage::ReadStorage;

use crate::error::Error;
use crate::types;
use crate::types::cobs;
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

#[tauri::command]
pub fn repo_by_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
) -> Result<types::repo::RepoInfo, Error> {
    let (repo, doc) = ctx.repo(rid)?;
    let repo_info = ctx.repo_info(&repo, doc)?;

    Ok::<_, Error>(repo_info)
}

#[tauri::command]
pub async fn diff(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: String,
    head: String,
) -> Result<cobs::Stats, Error> {
    let (repo, _) = ctx.repo(rid)?;
    let repo = radicle_surf::Repository::open(repo.path())?;
    let base = repo.commit(base)?;
    let commit = repo.commit(head)?;
    let diff = repo.diff(base.id, commit.id)?;

    let stats = diff.stats();

    Ok::<_, Error>(cobs::Stats::new(stats))
}
