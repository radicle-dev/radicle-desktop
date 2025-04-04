use axum::routing::post;
use axum::Router;

use handlers::{
    comment_threads_by_issue_handler, get_embed_handler, save_embed_by_bytes_handler,
    save_embed_by_clipboard_handler, save_embed_by_path_handler, save_embed_to_disk_handler,
};

use crate::registry::StateRegistry;

pub mod handlers;
pub mod models;

pub fn router() -> Router<StateRegistry> {
    Router::new()
        .route("/get_embed", post(get_embed_handler))
        .route("/save_embed_by_path", post(save_embed_by_path_handler))
        .route(
            "/save_embed_by_clipboard",
            post(save_embed_by_clipboard_handler),
        )
        .route("/save_embed_by_bytes", post(save_embed_by_bytes_handler))
        .route("/save_embed_to_disk", post(save_embed_to_disk_handler))
        .route(
            "/comment_threads_by_issue_id",
            post(comment_threads_by_issue_handler),
        )
}
