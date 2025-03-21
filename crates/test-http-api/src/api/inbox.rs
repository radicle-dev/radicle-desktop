use axum::{routing::post, Router};

use super::inbox::handlers::{
    clear_notifications_handler, count_notifications_by_repo_handler, list_notifications_handler,
};

use crate::registry::StateRegistry;

pub mod handlers;
pub mod models;

pub fn router() -> Router<StateRegistry> {
    Router::new()
        .route("/list_notifications", post(list_notifications_handler))
        .route(
            "/count_notifications_by_repo",
            post(count_notifications_by_repo_handler),
        )
        .route("/clear_notifications", post(clear_notifications_handler))
}
