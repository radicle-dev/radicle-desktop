use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use radicle_types::domain::repo::models::cobs;
use radicle_types::domain::repo::service::Service;
use radicle_types::domain::repo::traits::RepoService as _;
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle;
use radicle_types::outbound::sqlite::Sqlite;

use crate::registry::StateRegistry;

use super::models::{
    ActivityBody, CreateIssueCommentPayload, CreateIssuePayload, EditIssuePayload, IssueBody,
    IssuesBody,
};

pub(crate) async fn create_issue_comment_handler(
    State(app_state): State<StateRegistry>,
    Json(CreateIssueCommentPayload { rid, new, opts }): Json<CreateIssueCommentPayload>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let commits = service.create_issue_comment(rid, new, opts)?;

    Ok::<_, Error>(Json(commits))
}

pub(crate) async fn edit_issue_handler(
    State(app_state): State<StateRegistry>,
    Json(EditIssuePayload {
        rid,
        cob_id,
        action,
        opts,
    }): Json<EditIssuePayload>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let issue = service.edit_issue(rid, cob_id.into(), action, opts)?;

    Ok::<_, Error>(Json(issue))
}

pub(crate) async fn create_issue_handler(
    State(app_state): State<StateRegistry>,
    Json(CreateIssuePayload { rid, new, opts }): Json<CreateIssuePayload>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let issue = service.create_issue(rid, new, opts)?;

    Ok::<_, Error>(Json(issue))
}

pub(crate) async fn issues_handler(
    State(app_state): State<StateRegistry>,
    Json(IssuesBody { rid, status }): Json<IssuesBody>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let issues = service.list_issues(rid, status)?;

    Ok::<_, Error>(Json(issues))
}

pub(crate) async fn issue_handler(
    State(app_state): State<StateRegistry>,
    Json(IssueBody { rid, id }): Json<IssueBody>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let issue = service.issue_by_id(rid, id)?;

    Ok::<_, Error>(Json(issue))
}

pub(crate) async fn activity_issue_handler(
    State(app_state): State<StateRegistry>,
    Json(ActivityBody { rid, id }): Json<ActivityBody>,
) -> impl IntoResponse {
    let service = app_state.state::<Service<Radicle, Sqlite>>().await.unwrap();
    let activity = service.activity_by_id::<radicle::issue::Action, cobs::issue::Action>(
        rid,
        &radicle::cob::issue::TYPENAME,
        id,
    )?;

    Ok::<_, Error>(Json(activity))
}
