use axum::{extract::State, response::IntoResponse, Json};

use radicle::Profile;
use radicle_types::domain::inbox::service::Service as InboxService;
use radicle_types::domain::inbox::traits::InboxService as _;
use radicle_types::error::Error;
use radicle_types::outbound::sqlite::Sqlite;

use crate::registry::StateRegistry;

use super::models::{ClearNotificationsPayload, ListNotificationsPayload};

pub(crate) async fn list_notifications_handler(
    State(app_state): State<StateRegistry>,
    Json(ListNotificationsPayload { params }): Json<ListNotificationsPayload>,
) -> impl IntoResponse {
    let profile = app_state.state::<Profile>().await.unwrap();
    let service = app_state.state::<InboxService<Sqlite>>().await.unwrap();
    let result = service.list_notifications(&profile, params)?;

    Ok::<_, Error>(Json(result))
}

pub(crate) async fn count_notifications_by_repo_handler(
    State(app_state): State<StateRegistry>,
) -> impl IntoResponse {
    let profile = app_state.state::<Profile>().await.unwrap();
    let service = app_state.state::<InboxService<Sqlite>>().await.unwrap();
    let result = service.count_notifications_by_repo(profile.storage.clone())?;

    Ok::<_, Error>(Json(result))
}

pub(crate) async fn clear_notifications_handler(
    State(app_state): State<StateRegistry>,
    Json(ClearNotificationsPayload { params }): Json<ClearNotificationsPayload>,
) -> impl IntoResponse {
    let profile = app_state.state::<Profile>().await.unwrap();
    let service = app_state.state::<InboxService<Sqlite>>().await.unwrap();
    let result = service.clear_notifications(&profile, params);

    Ok::<_, Error>(Json(result))
}
