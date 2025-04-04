use axum::routing::post;
use axum::Router;

use handlers::{
    activity_issue_handler, create_issue_comment_handler, create_issue_handler, edit_issue_handler,
    issue_handler, issues_handler,
};

use crate::registry::StateRegistry;

pub mod handlers;
pub mod models;

pub fn router() -> Router<StateRegistry> {
    Router::new()
        .route("/create_issue_comment", post(create_issue_comment_handler))
        .route("/create_issue", post(create_issue_handler))
        .route("/edit_issue", post(edit_issue_handler))
        .route("/list_issues", post(issues_handler))
        .route("/activity_by_issue", post(activity_issue_handler))
        .route("/issue_by_id", post(issue_handler))
}
