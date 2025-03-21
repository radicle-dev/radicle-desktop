use radicle::git;
use radicle::identity::RepoId;

use radicle_types::domain::repo::models::cobs;
use radicle_types::domain::repo::models::repo;
use radicle_types::domain::repo::service::Service;
use radicle_types::domain::repo::traits::RepoService;
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
    rid: RepoId,
) -> Result<repo::RepoInfo, Error> {
    service.repo_by_id(rid)
}

#[tauri::command]
pub async fn diff_stats(
    service: tauri::State<'_, Service<Radicle, Sqlite>>,
    rid: RepoId,
    base: git::Oid,
    head: git::Oid,
) -> Result<cobs::Stats, Error> {
    service.diff_stats(rid, base, head)
}

#[tauri::command]
pub async fn list_commits(
    service: tauri::State<'_, Service<Radicle, Sqlite>>,
    rid: RepoId,
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
    service.create_repo(name, description)
}
