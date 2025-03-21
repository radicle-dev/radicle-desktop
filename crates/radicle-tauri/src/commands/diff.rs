use radicle::identity;
use radicle_types::domain::repo::models;
use radicle_types::domain::repo::service::Service;
use radicle_types::domain::repo::traits::RepoService;
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle as R;
use radicle_types::outbound::sqlite::Sqlite;

#[tauri::command]
pub async fn get_diff(
    service: tauri::State<'_, Service<R, Sqlite>>,
    rid: identity::RepoId,
    options: models::diff::Options,
) -> Result<models::diff::Diff, Error> {
    service.get_diff(rid, options)
}
