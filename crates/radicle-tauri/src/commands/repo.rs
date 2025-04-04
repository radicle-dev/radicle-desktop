use radicle::{git, identity};

use radicle_types::domain::repo::models::{diff, repo};
use radicle_types::domain::repo::service::Service;
use radicle_types::domain::repo::traits::RepoService as _;
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle;
use radicle_types::outbound::sqlite::Sqlite;

#[tauri::command]
pub fn list_repos(
    service: tauri::State<Service<Radicle, Sqlite>>,
    show: repo::Show,
) -> Result<Vec<repo::RepoInfo>, Error> {
    service.list_repos(show)
}

#[tauri::command]
pub fn repo_count(
    service: tauri::State<Service<Radicle, Sqlite>>,
) -> Result<repo::RepoCount, Error> {
    service.repo_count()
}

#[tauri::command]
pub fn repo_by_id(
    service: tauri::State<Service<Radicle, Sqlite>>,
    rid: identity::RepoId,
) -> Result<repo::RepoInfo, Error> {
    service.repo_by_id(rid)
}

#[tauri::command]
pub async fn diff_stats(
    service: tauri::State<'_, Service<Radicle, Sqlite>>,
    rid: identity::RepoId,
    base: git::Oid,
    head: git::Oid,
) -> Result<diff::Stats, Error> {
    service.diff_stats(rid, base, head)
}

#[tauri::command]
pub async fn list_commits(
    service: tauri::State<'_, Service<Radicle, Sqlite>>,
    rid: identity::RepoId,
    base: git::Oid,
    head: git::Oid,
) -> Result<Vec<repo::Commit>, Error> {
    service.list_commits(rid, base, head)
}

#[tauri::command]
pub(crate) async fn create_repo(
    service: tauri::State<'_, Service<Radicle, Sqlite>>,
    name: String,
    description: String,
) -> Result<(), Error> {
    let config = radicle::git::raw::Config::open_default()?;
    // SAFETY: "master" is always a valid RefString
    let default_branch = git::RefString::try_from(
        config
            .get_string("init.defaultBranch")
            .unwrap_or("master".to_owned()),
    )
    .unwrap();

    service.create_repo(name, description, default_branch, None)
}
