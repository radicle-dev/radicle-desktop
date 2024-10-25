use std::ops::Deref;
use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Json};
use axum::routing::post;
use axum::Router;
use hyper::header::CONTENT_TYPE;
use hyper::Method;
use radicle::cob::TypeName;
use serde::{Deserialize, Serialize};
use tower_http::cors::{self, CorsLayer};

use radicle::{git, identity};
use radicle_types as types;
use radicle_types::cobs::issue::{Action, NewIssue};
use radicle_types::cobs::CobOptions;
use radicle_types::traits::auth::Auth;
use radicle_types::traits::cobs::Cobs;
use radicle_types::traits::issue::{Issues, IssuesMut};
use radicle_types::traits::patch::{Patches, PatchesMut};
use radicle_types::traits::repo::Repo;
use radicle_types::traits::thread::Thread;
use radicle_types::traits::Profile;

use crate::error::Error;

#[derive(Clone)]
pub struct Context {
    profile: Arc<radicle::Profile>,
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
    pub fn new(profile: Arc<radicle::Profile>) -> Self {
        Self { profile }
    }
}

pub fn router(ctx: Context) -> Router {
    Router::new()
        .route("/config", post(config_handler))
        .route("/authenticate", post(auth_handler))
        .route("/list_repos", post(repo_root_handler))
        .route("/repo_by_id", post(repo_handler))
        .route("/diff_stats", post(diff_stats_handler))
        .route("/activity_by_id", post(activity_handler))
        .route("/get_diff", post(diff_handler))
        .route("/list_issues", post(issues_handler))
        .route("/create_issue", post(create_issue_handler))
        .route("/edit_issue", post(edit_issue_handler))
        .route("/issue_by_id", post(issue_handler))
        .route("/list_patches", post(patches_handler))
        .route("/patch_by_id", post(patch_handler))
        .route("/revisions_by_patch", post(revision_handler))
        .route("/get_file_by_oid", post(get_embeds_handler))
        .route("/save_embed", post(save_embed_handler))
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

async fn repo_root_handler(State(ctx): State<Context>) -> impl IntoResponse {
    let repos = ctx.list_repos()?;

    Ok::<_, Error>(Json(repos))
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
#[serde(rename_all = "camelCase")]
struct EditIssuesBody {
    pub rid: identity::RepoId,
    pub cob_id: git::Oid,
    pub action: Action,
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
    pub type_name: TypeName,
    pub id: git::Oid,
}

async fn activity_handler(
    State(ctx): State<Context>,
    Json(ActivityBody { rid, type_name, id }): Json<ActivityBody>,
) -> impl IntoResponse {
    let activity = ctx.activity_by_id(rid, type_name, id)?;

    Ok::<_, Error>(Json(activity))
}

#[derive(Serialize, Deserialize)]
struct EmbedBody {
    pub rid: identity::RepoId,
    pub oid: git::Oid,
}

async fn get_embeds_handler(
    State(ctx): State<Context>,
    Json(EmbedBody { rid, oid }): Json<EmbedBody>,
) -> impl IntoResponse {
    let embed = ctx.get_embed(rid, oid)?;

    Ok::<_, Error>(Json(embed))
}

#[derive(Serialize, Deserialize)]
struct CreateEmbedBody {
    pub rid: identity::RepoId,
    pub name: String,
    pub content: Vec<u8>,
}

async fn save_embed_handler(
    State(ctx): State<Context>,
    Json(CreateEmbedBody { rid, name, content }): Json<CreateEmbedBody>,
) -> impl IntoResponse {
    let embed = ctx.save_embed(rid, &name, &content)?;

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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PatchesBody {
    pub rid: identity::RepoId,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub status: Option<types::cobs::query::PatchStatus>,
}

async fn patches_handler(
    State(ctx): State<Context>,
    Json(PatchesBody {
        rid,
        page,
        per_page,
        status,
    }): Json<PatchesBody>,
) -> impl IntoResponse {
    let patches = ctx.list_patches(rid, status, page, per_page)?;

    Ok::<_, Error>(Json(patches))
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