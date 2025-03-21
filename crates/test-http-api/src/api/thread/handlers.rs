use axum::{extract::State, response::IntoResponse, Json};

use radicle_types::domain::repo::service::Service as RepoService;
use radicle_types::domain::repo::traits::RepoService as _;
use radicle_types::error::Error;
use radicle_types::outbound::{radicle::Radicle, sqlite::Sqlite};

use crate::api::issue::models::IssueBody;
use crate::registry::StateRegistry;

use super::models::{CreateEmbedBody, GetEmbedPayload};

pub(crate) async fn save_embed_handler(
    State(app_state): State<StateRegistry>,
    Json(CreateEmbedBody { rid, path }): Json<CreateEmbedBody>,
) -> impl IntoResponse {
    let service = app_state
        .state::<RepoService<Radicle, Sqlite>>()
        .await
        .unwrap();
    let embed = service.save_embed_by_path(rid, path)?;

    Ok::<_, Error>(Json(embed))
}

pub(crate) async fn get_embed_handler(
    State(app_state): State<StateRegistry>,
    Json(GetEmbedPayload { rid, name, oid }): Json<GetEmbedPayload>,
) -> impl IntoResponse {
    let service = app_state
        .state::<RepoService<Radicle, Sqlite>>()
        .await
        .unwrap();
    let embed = service.get_embed(rid, name, oid)?;

    Ok::<_, Error>(Json(embed))
}

pub(crate) async fn comment_threads_by_issue_handler(
    State(app_state): State<StateRegistry>,
    Json(IssueBody { rid, id }): Json<IssueBody>,
) -> impl IntoResponse {
    let service = app_state
        .state::<RepoService<Radicle, Sqlite>>()
        .await
        .unwrap();
    let threads = service.comment_threads_by_issue_id(rid, id)?;

    Ok::<_, Error>(Json(threads))
}
