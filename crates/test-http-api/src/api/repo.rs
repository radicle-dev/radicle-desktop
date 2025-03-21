use axum::{routing::post, Router};
use handlers::{
    create_repo_handler, diff_stats_handler, get_diff_handler, list_commits_handler,
    repo_count_handler, repo_handler, repo_root_handler,
};

use crate::registry::StateRegistry;

pub mod handlers;
pub mod models;

pub fn router() -> Router<StateRegistry> {
    Router::new()
        .route("/diff_stats", post(diff_stats_handler))
        .route("/get_diff", post(get_diff_handler))
        .route("/list_commits", post(list_commits_handler))
        .route("/list_repos", post(repo_root_handler))
        .route("/repo_by_id", post(repo_handler))
        .route("/repo_count", post(repo_count_handler))
        .route("/create_repo", post(create_repo_handler))
}
