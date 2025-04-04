use std::env;

use arboard::Clipboard;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use radicle_types::domain::repo::service::Service;
use radicle_types::domain::repo::traits::RepoService as _;
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle;
use radicle_types::outbound::sqlite::Sqlite;

use crate::api::issue::models::IssueBody;
use crate::registry::StateRegistry;

use super::models::{
    CreateEmbedBodyByBytes, CreateEmbedBodyByClipboard, CreateEmbedBodyByPath, GetEmbedPayload,
    SaveEmbedToDisk,
};

pub(crate) async fn save_embed_by_path_handler(
    State(app_state): State<StateRegistry>,
    Json(CreateEmbedBodyByPath { rid, path }): Json<CreateEmbedBodyByPath>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let embed = service.save_embed_by_path(rid, path)?;

    Ok::<_, Error>(Json(embed))
}

pub(crate) async fn save_embed_by_bytes_handler(
    State(app_state): State<StateRegistry>,
    Json(CreateEmbedBodyByBytes { rid, name, bytes }): Json<CreateEmbedBodyByBytes>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let embed = service.save_embed_by_bytes(rid, name, bytes)?;

    Ok::<_, Error>(Json(embed))
}

pub(crate) async fn save_embed_by_clipboard_handler(
    State(app_state): State<StateRegistry>,
    Json(CreateEmbedBodyByClipboard { rid, name }): Json<CreateEmbedBodyByClipboard>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let mut clipboard = Clipboard::new().expect("Clipboards are not supported on this platform.");
    let image = clipboard.get_image().expect("Clipboard is empty, contents are not an image, or the contents cannot be converted to an appropriate format.");
    let embed = service.save_embed_by_bytes(rid, name, image.into_owned_bytes().to_vec())?;

    Ok::<_, Error>(Json(embed))
}

/// Note for testing, we don't have a file dialog, but we try to hardcode the name into a test path
pub(crate) async fn save_embed_to_disk_handler(
    State(app_state): State<StateRegistry>,
    Json(SaveEmbedToDisk { rid, oid, name }): Json<SaveEmbedToDisk>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let path = env::current_dir()?;
    service.save_embed_to_disk(rid, oid, path.join(name))?;

    Ok::<_, Error>(Json(()))
}

pub(crate) async fn get_embed_handler(
    State(app_state): State<StateRegistry>,
    Json(GetEmbedPayload { rid, name, oid }): Json<GetEmbedPayload>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let embed = service.get_embed(rid, name, oid)?;

    Ok::<_, Error>(Json(embed))
}

pub(crate) async fn comment_threads_by_issue_handler(
    State(app_state): State<StateRegistry>,
    Json(IssueBody { rid, id }): Json<IssueBody>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let threads = service.comment_threads_by_issue_id(rid, id)?;

    Ok::<_, Error>(Json(threads))
}
