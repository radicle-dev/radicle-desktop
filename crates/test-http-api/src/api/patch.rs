use axum::{routing::post, Router};
use handlers::{
    activity_patch_handler, edit_patch_handler, patch_handler, patches_handler,
    review_by_patch_and_revision_and_id_handler, revisions_by_patch_handler,
};

use crate::registry::StateRegistry;

pub mod handlers;
pub mod models;

pub fn router() -> Router<StateRegistry> {
    Router::new()
        .route(
            "/review_by_patch_and_revision_and_id",
            post(review_by_patch_and_revision_and_id_handler),
        )
        .route("/edit_patch", post(edit_patch_handler))
        .route("/activity_by_patch", post(activity_patch_handler))
        .route("/revisions_by_patch", post(revisions_by_patch_handler))
        .route("/list_patches", post(patches_handler))
        .route("/patch_by_id", post(patch_handler))
}
