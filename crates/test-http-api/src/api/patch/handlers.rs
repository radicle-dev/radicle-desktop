use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use radicle::Profile;

use radicle_types::domain::repo::models::cobs;
use radicle_types::domain::repo::service::Service;
use radicle_types::domain::repo::traits::RepoService as _;
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle;
use radicle_types::outbound::sqlite::Sqlite;

use crate::registry::StateRegistry;

use super::models::{
    ActivityBody, EditPatchPayload, PatchBody, PatchRevisionsPayload, PatchesBody,
    ReviewByPatchPayload,
};

pub(crate) async fn revisions_by_patch_handler(
    State(app_state): State<StateRegistry>,
    Json(PatchRevisionsPayload { rid, id }): Json<PatchRevisionsPayload>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let result = service.revisions_by_patch(rid, id)?;

    Ok::<_, Error>(Json(result))
}

pub(crate) async fn edit_patch_handler(
    State(app_state): State<StateRegistry>,
    Json(EditPatchPayload {
        rid,
        cob_id,
        action,
        opts,
    }): Json<EditPatchPayload>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let patch = service.edit_patch(rid, cob_id.into(), action, opts)?;

    Ok::<_, Error>(Json(patch))
}

pub(crate) async fn review_by_patch_and_revision_and_id_handler(
    State(app_state): State<StateRegistry>,
    Json(ReviewByPatchPayload {
        rid,
        id,
        revision_id,
        review_id,
    }): Json<ReviewByPatchPayload>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let review = service.review_by_id(rid, id, revision_id, review_id)?;

    Ok::<_, Error>(Json(review))
}

pub(crate) async fn activity_patch_handler(
    State(app_state): State<StateRegistry>,
    Json(ActivityBody { rid, id }): Json<ActivityBody>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let activity = service.activity_by_id::<radicle::patch::Action, cobs::patch::Action>(
        rid,
        &radicle::cob::patch::TYPENAME,
        id,
    )?;

    Ok::<_, Error>(Json(activity))
}

pub(crate) async fn patches_handler(
    State(app_state): State<StateRegistry>,
    Json(PatchesBody {
        rid,
        skip,
        take,
        status,
    }): Json<PatchesBody>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let profile = app_state.state::<Profile>().await.unwrap();
    let patches = match status {
        Some(status) => service
            .list_patches_by_status(rid, status.into())?
            .collect::<Vec<_>>(),
        None => service.list_patches(rid)?.collect::<Vec<_>>(),
    };
    let total_count = patches.len();

    Ok::<_, Error>(Json(
        cobs::PaginatedQuery::<Vec<cobs::patch::Patch>>::map_with_pagination(
            patches.into_iter(),
            total_count,
            skip.unwrap_or(0),
            take.unwrap_or(20),
            |(id, patch)| cobs::patch::Patch::new(&id, &patch, &profile.aliases()),
        ),
    ))
}

pub(crate) async fn patch_handler(
    State(app_state): State<StateRegistry>,
    Json(PatchBody { rid, id }): Json<PatchBody>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let patch = service.get_patch_by_id(rid, id.into())?;

    Ok::<_, Error>(Json(patch))
}
