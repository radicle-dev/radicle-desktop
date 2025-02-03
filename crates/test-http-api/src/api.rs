use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Json};
use axum::routing::post;
use axum::Router;
use hyper::header::CONTENT_TYPE;
use hyper::Method;
use serde::{Deserialize, Serialize};
use tower_http::cors::{self, CorsLayer};

use radicle::{git, identity};
use radicle_types as types;
use radicle_types::cobs;
use radicle_types::cobs::issue;
use radicle_types::cobs::issue::NewIssue;
use radicle_types::cobs::CobOptions;
use radicle_types::domain::inbox::models::notification::NotificationCount;
use radicle_types::domain::patch::models;
use radicle_types::domain::patch::service::Service;
use radicle_types::domain::patch::traits::PatchService;
use radicle_types::error::Error;
use radicle_types::outbound::sqlite::Sqlite;
use radicle_types::traits::auth::Auth;
use radicle_types::traits::cobs::Cobs;
use radicle_types::traits::issue::{Issues, IssuesMut};
use radicle_types::traits::patch::{Patches, PatchesMut};
use radicle_types::traits::repo::{Repo, Show};
use radicle_types::traits::thread::Thread;
use radicle_types::traits::Profile;

#[derive(Clone)]
pub struct Context {
    profile: Arc<radicle::Profile>,
    patches: Arc<Service<Sqlite>>,
}

impl Auth for Context {}
impl Repo for Context {}
impl Cobs for Context {}
impl Thread for Context {}
impl Issues for Context {}
impl IssuesMut for Context {}
impl Patches for Context {}
impl PatchesMut for Context {}
impl Profile for Context {
    fn profile(&self) -> radicle::Profile {
        self.profile.deref().clone()
    }
}

impl Context {
    pub fn new(profile: Arc<radicle::Profile>, patches: Arc<Service<Sqlite>>) -> Self {
        Self { profile, patches }
    }
}

pub fn router(ctx: Context) -> Router {
    Router::new()
        .route("/config", post(config_handler))
        .route("/authenticate", post(auth_handler))
        .route(
            "/count_notifications_by_repo",
            post(repo_count_notifications_handler),
        )
        .route("/repo_count", post(repo_count_handler))
        .route("/list_repos", post(repo_root_handler))
        .route("/repo_by_id", post(repo_handler))
        .route("/diff_stats", post(diff_stats_handler))
        .route(
            "/activity_by_issue",
            post(activity_issue_handler::<issue::Action>),
        )
        .route(
            "/activity_by_patch",
            post(activity_patch_handler::<models::patch::Action>),
        )
        .route("/get_diff", post(diff_handler))
        .route("/list_issues", post(issues_handler))
        .route("/create_issue", post(create_issue_handler))
        .route("/create_issue_comment", post(create_issue_comment_handler))
        .route("/edit_issue", post(edit_issue_handler))
        .route("/issue_by_id", post(issue_handler))
        .route("/comment_threads_by_issue_id", post(issue_threads_handler))
        .route("/list_patches", post(patches_handler))
        .route("/patch_by_id", post(patch_handler))
        .route("/revisions_by_patch", post(revision_handler))
        .route("/get_embed", post(get_embeds_handler))
        .route("/save_embed_by_path", post(save_embed_handler))
        .route("/save_embed_by_clipboard", post(save_embed_handler))
        .route("/save_embed_by_bytes", post(save_embed_handler))
        .route("/save_embed_to_disk", post(save_embed_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(cors::Any)
                .allow_methods([Method::POST, Method::GET])
                .allow_headers([CONTENT_TYPE]),
        )
        .with_state(ctx)
}

async fn config_handler(State(ctx): State<Context>) -> impl IntoResponse {
    let config = ctx.config();

    Ok::<_, Error>(Json(config))
}

async fn auth_handler(State(ctx): State<Context>) -> impl IntoResponse {
    ctx.authenticate()?;

    Ok::<_, Error>(Json(()))
}

#[derive(Serialize, Deserialize)]
pub struct Options {
    show: Show,
}

async fn repo_root_handler(
    State(ctx): State<Context>,
    Json(Options { show }): Json<Options>,
) -> impl IntoResponse {
    let repos = ctx.list_repos(show)?;

    Ok::<_, Error>(Json(repos))
}

async fn repo_count_handler(State(ctx): State<Context>) -> impl IntoResponse {
    let repos = ctx.repo_count()?;
    Ok::<_, Error>(Json(repos))
}

async fn repo_count_notifications_handler() -> impl IntoResponse {
    Ok::<_, Error>(Json(Vec::<NotificationCount>::new()))
}

#[derive(Serialize, Deserialize)]
struct RepoBody {
    pub rid: identity::RepoId,
}

async fn repo_handler(
    State(ctx): State<Context>,
    Json(RepoBody { rid }): Json<RepoBody>,
) -> impl IntoResponse {
    let info = ctx.repo_by_id(rid)?;

    Ok::<_, Error>(Json(info))
}

#[derive(Serialize, Deserialize)]
struct DiffStatsBody {
    pub rid: identity::RepoId,
    pub base: git::Oid,
    pub head: git::Oid,
}

async fn diff_stats_handler(
    State(ctx): State<Context>,
    Json(DiffStatsBody { rid, base, head }): Json<DiffStatsBody>,
) -> impl IntoResponse {
    let info = ctx.diff_stats(rid, base, head)?;

    Ok::<_, Error>(Json(info))
}

#[derive(Serialize, Deserialize)]
struct DiffBody {
    pub rid: identity::RepoId,
    pub options: types::cobs::diff::Options,
}

async fn diff_handler(
    State(ctx): State<Context>,
    Json(DiffBody { rid, options }): Json<DiffBody>,
) -> impl IntoResponse {
    let info = ctx.get_diff(rid, options)?;

    Ok::<_, Error>(Json(info))
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IssuesBody {
    pub rid: identity::RepoId,
    pub status: Option<types::cobs::query::IssueStatus>,
}

async fn issues_handler(
    State(ctx): State<Context>,
    Json(IssuesBody { rid, status }): Json<IssuesBody>,
) -> impl IntoResponse {
    let issues = ctx.list_issues(rid, status)?;

    Ok::<_, Error>(Json(issues))
}

#[derive(Serialize, Deserialize)]
struct CreateIssuesBody {
    pub rid: identity::RepoId,
    pub opts: CobOptions,
    pub new: NewIssue,
}

async fn create_issue_handler(
    State(ctx): State<Context>,
    Json(CreateIssuesBody { rid, opts, new }): Json<CreateIssuesBody>,
) -> impl IntoResponse {
    let issues = ctx.create_issue(rid, new, opts)?;

    Ok::<_, Error>(Json(issues))
}

#[derive(Serialize, Deserialize)]
struct CreateIssueCommentBody {
    pub rid: identity::RepoId,
    pub new: types::cobs::thread::NewIssueComment,
    pub opts: types::cobs::CobOptions,
}

async fn create_issue_comment_handler(
    State(ctx): State<Context>,
    Json(CreateIssueCommentBody { rid, opts, new }): Json<CreateIssueCommentBody>,
) -> impl IntoResponse {
    let comment = ctx.create_issue_comment(rid, new, opts)?;

    Ok::<_, Error>(Json(comment))
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditIssuesBody {
    pub rid: identity::RepoId,
    pub cob_id: git::Oid,
    pub action: issue::Action,
    pub opts: CobOptions,
}

async fn edit_issue_handler(
    State(ctx): State<Context>,
    Json(EditIssuesBody {
        rid,
        cob_id,
        action,
        opts,
    }): Json<EditIssuesBody>,
) -> impl IntoResponse {
    let issues = ctx.edit_issue(rid, cob_id, action, opts)?;

    Ok::<_, Error>(Json(issues))
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivityBody {
    pub rid: identity::RepoId,
    pub id: git::Oid,
}

async fn activity_issue_handler<A: serde::Serialize + serde::de::DeserializeOwned>(
    State(ctx): State<Context>,
    Json(ActivityBody { rid, id }): Json<ActivityBody>,
) -> impl IntoResponse {
    let activity = ctx.activity_by_id::<A>(rid, &radicle::cob::issue::TYPENAME, id)?;

    Ok::<_, Error>(Json(activity))
}

async fn activity_patch_handler<A: serde::Serialize + serde::de::DeserializeOwned>(
    State(ctx): State<Context>,
    Json(ActivityBody { rid, id }): Json<ActivityBody>,
) -> impl IntoResponse {
    let activity = ctx.activity_by_id::<A>(rid, &radicle::cob::patch::TYPENAME, id)?;

    Ok::<_, Error>(Json(activity))
}

#[derive(Serialize, Deserialize)]
struct EmbedBody {
    pub rid: identity::RepoId,
    pub name: Option<String>,
    pub oid: git::Oid,
}

async fn get_embeds_handler(
    State(ctx): State<Context>,
    Json(EmbedBody { rid, name, oid }): Json<EmbedBody>,
) -> impl IntoResponse {
    let embed = ctx.get_embed(rid, name, oid)?;

    Ok::<_, Error>(Json(embed))
}

#[derive(Serialize, Deserialize)]
struct CreateEmbedBody {
    pub rid: identity::RepoId,
    pub path: PathBuf,
}

async fn save_embed_handler(
    State(ctx): State<Context>,
    Json(CreateEmbedBody { rid, path }): Json<CreateEmbedBody>,
) -> impl IntoResponse {
    let embed = ctx.save_embed_by_path(rid, path)?;

    Ok::<_, Error>(Json(embed))
}

#[derive(Serialize, Deserialize)]
struct IssueBody {
    pub rid: identity::RepoId,
    pub id: git::Oid,
}

async fn issue_handler(
    State(ctx): State<Context>,
    Json(IssueBody { rid, id }): Json<IssueBody>,
) -> impl IntoResponse {
    let issue = ctx.issue_by_id(rid, id)?;

    Ok::<_, Error>(Json(issue))
}

async fn issue_threads_handler(
    State(ctx): State<Context>,
    Json(IssueBody { rid, id }): Json<IssueBody>,
) -> impl IntoResponse {
    let issue_threads = ctx.comment_threads_by_issue_id(rid, id)?;

    Ok::<_, Error>(Json(issue_threads))
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PatchesBody {
    pub rid: identity::RepoId,
    pub skip: Option<usize>,
    pub take: Option<usize>,
    pub status: Option<types::cobs::query::PatchStatus>,
}

async fn patches_handler(
    State(ctx): State<Context>,
    Json(PatchesBody {
        rid,
        skip,
        take,
        status,
    }): Json<PatchesBody>,
) -> impl IntoResponse {
    let profile = ctx.profile;
    let cursor = skip.unwrap_or(0);
    let take = take.unwrap_or(20);
    let aliases = profile.aliases();
    let patches = match status {
        None => ctx.patches.list(rid)?.collect::<Vec<_>>(),
        Some(s) => ctx
            .patches
            .list_by_status(rid, s.into())?
            .collect::<Vec<_>>(),
    };
    let more = cursor + take < patches.len();

    let patches = patches
        .into_iter()
        .map(|(id, patch)| models::patch::Patch::new(id, &patch, &aliases))
        .skip(cursor)
        .take(take)
        .collect::<Vec<_>>();

    Ok::<_, Error>(Json(cobs::PaginatedQuery {
        cursor,
        more,
        content: patches,
    }))
}

#[derive(Serialize, Deserialize)]
struct PatchBody {
    pub rid: identity::RepoId,
    pub id: git::Oid,
}

async fn patch_handler(
    State(ctx): State<Context>,
    Json(PatchBody { rid, id }): Json<PatchBody>,
) -> impl IntoResponse {
    let patch = ctx.get_patch(rid, id)?;

    Ok::<_, Error>(Json(patch))
}

async fn revision_handler(
    State(ctx): State<Context>,
    Json(PatchBody { rid, id }): Json<PatchBody>,
) -> impl IntoResponse {
    let revisions = ctx.revisions_by_patch(rid, id)?;

    Ok::<_, Error>(Json(revisions))
}
