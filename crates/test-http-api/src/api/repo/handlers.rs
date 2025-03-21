use axum::{extract::State, response::IntoResponse, Json};
use radicle_types::domain::repo::service::Service as RepoService;
use radicle_types::domain::repo::traits::RepoService as _;
use radicle_types::error::Error;
use radicle_types::outbound::{radicle::Radicle, sqlite::Sqlite};

use crate::registry::StateRegistry;

use super::models::{CreateRepoPayload, DiffPayload, GetDiffPayload, RepoPayload, RepoRootOptions};

pub(crate) async fn diff_stats_handler(
    State(app_state): State<StateRegistry>,
    Json(DiffPayload { rid, base, head }): Json<DiffPayload>,
) -> impl IntoResponse {
    let service = app_state
        .state::<RepoService<Radicle, Sqlite>>()
        .await
        .unwrap();
    let stats = service.diff_stats(rid, base, head)?;

    Ok::<_, Error>(Json(stats))
}

pub(crate) async fn list_commits_handler(
    State(app_state): State<StateRegistry>,
    Json(DiffPayload { rid, base, head }): Json<DiffPayload>,
) -> impl IntoResponse {
    let service = app_state
        .state::<RepoService<Radicle, Sqlite>>()
        .await
        .unwrap();
    let commits = service.list_commits(rid, base, head)?;

    Ok::<_, Error>(Json(commits))
}

pub(crate) async fn get_diff_handler(
    State(app_state): State<StateRegistry>,
    Json(GetDiffPayload { rid, options }): Json<GetDiffPayload>,
) -> impl IntoResponse {
    let service = app_state
        .state::<RepoService<Radicle, Sqlite>>()
        .await
        .unwrap();
    let diff = service.get_diff(rid, options)?;

    Ok::<_, Error>(Json(diff))
}

pub(crate) async fn repo_root_handler(
    State(app_state): State<StateRegistry>,
    Json(RepoRootOptions { show }): Json<RepoRootOptions>,
) -> impl IntoResponse {
    let service = app_state
        .state::<RepoService<Radicle, Sqlite>>()
        .await
        .unwrap();
    let repos = (*service).clone().list_repos(show)?;

    Ok::<_, Error>(Json(repos))
}

pub(crate) async fn repo_handler(
    State(app_state): State<StateRegistry>,
    Json(payload): Json<RepoPayload>,
) -> impl IntoResponse {
    let service = app_state
        .state::<RepoService<Radicle, Sqlite>>()
        .await
        .unwrap();
    let info = service.repo_by_id(payload.rid)?;

    Ok::<_, Error>(Json(info))
}

pub(crate) async fn repo_count_handler(
    State(app_state): State<StateRegistry>,
) -> impl IntoResponse {
    let service = app_state
        .state::<RepoService<Radicle, Sqlite>>()
        .await
        .unwrap();
    let count = service.repo_count()?;

    Ok::<_, Error>(Json(count))
}

pub(crate) async fn create_repo_handler(
    State(app_state): State<StateRegistry>,
    Json(CreateRepoPayload { name, description }): Json<CreateRepoPayload>,
) -> impl IntoResponse {
    let service = app_state
        .state::<RepoService<Radicle, Sqlite>>()
        .await
        .unwrap();
    service.create_repo(name, description)?;

    Ok::<_, Error>(Json(()))
}
