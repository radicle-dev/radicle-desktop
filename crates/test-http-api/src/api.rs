use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::response::{IntoResponse, Json};
use axum::routing::post;
use hyper::Method;
use hyper::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use tower_http::cors::{self, CorsLayer};

use radicle::node::NodeId;
use radicle::{git, identity};
use radicle_types as types;
use radicle_types::cobs::CobOptions;
use radicle_types::cobs::issue;
use radicle_types::cobs::issue::NewIssue;
use radicle_types::cobs::{self, FromRadicleAction};
use radicle_types::config::Version;
use radicle_types::domain::patch::models;
use radicle_types::domain::patch::service::Service;
use radicle_types::domain::patch::traits::PatchService;
use radicle_types::error::Error;
use radicle_types::outbound::sqlite::Sqlite;
use radicle_types::traits::Profile;
use radicle_types::traits::cobs::Cobs;
use radicle_types::traits::issue::{Issues, IssuesMut};
use radicle_types::traits::job::Jobs;
use radicle_types::traits::patch::{Patches, PatchesMut};
use radicle_types::traits::release::Releases;
use radicle_types::traits::release_mut::ReleasesMut;
use radicle_types::traits::repo::{Repo, Show};
use radicle_types::traits::thread::Thread;

#[derive(Clone)]
pub struct Context {
    profile: Arc<radicle::Profile>,
    patches: Arc<Service<Sqlite>>,
}

impl Repo for Context {}
impl Cobs for Context {}
impl Thread for Context {}
impl Issues for Context {}
impl IssuesMut for Context {}
impl Jobs for Context {}
impl Patches for Context {}
impl PatchesMut for Context {}
impl Releases for Context {}
impl ReleasesMut for Context {}
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
        .route("/repo_count", post(repo_count_handler))
        .route("/list_repos", post(repo_root_handler))
        .route("/list_repos_summary", post(list_repos_summary_handler))
        .route(
            "/seeded_not_replicated",
            post(seeded_not_replicated_handler),
        )
        .route("/repo_by_id", post(repo_handler))
        .route("/list_repo_refs", post(list_repo_refs_handler))
        .route("/version", post(version_handler))
        .route("/diff_stats", post(diff_stats_handler))
        .route(
            "/activity_by_issue",
            post(activity_issue_handler::<radicle::issue::Action, issue::Action>),
        )
        .route(
            "/activity_by_patch",
            post(activity_patch_handler::<radicle::patch::Action, models::patch::Action>),
        )
        .route("/repo_readme", post(readme_handler))
        .route("/repo_tree", post(tree_handler))
        .route("/repo_blob", post(blob_handler))
        .route("/get_diff", post(diff_handler))
        .route("/get_commit_diff", post(commit_diff_handler))
        .route("/list_repo_commits", post(list_repo_commits_handler))
        .route("/repo_commit_count", post(repo_commit_count_handler))
        .route("/repo_commit", post(repo_commit_handler))
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
        .route("/list_jobs", post(jobs_handler))
        .route("/list_tags", post(list_tags_handler))
        .route("/list_releases", post(list_releases_handler))
        .route("/release_by_id", post(release_by_id_handler))
        .route("/releases_by_commit", post(releases_by_commit_handler))
        .route("/compute_artifact_cid", post(compute_artifact_cid_handler))
        .route(
            "/create_or_open_release",
            post(create_or_open_release_handler),
        )
        .route("/register_artifact", post(register_artifact_handler))
        // Stubs for the Tauri-only native picker and iroh seeding, so the
        // create-release flow is drivable without a dialog or running node.
        .route("/pick_artifact_files", post(pick_artifact_files_handler))
        .route("/seed_artifact", post(seed_artifact_handler))
        .route("/add_location", post(add_location_handler))
        .route("/remove_location", post(remove_location_handler))
        .route("/attest_artifact", post(attest_artifact_handler))
        .route("/set_metadata", post(set_metadata_handler))
        .route("/remove_metadata", post(remove_metadata_handler))
        .route("/redact_artifact", post(redact_artifact_handler))
        .route("/list_notifications", post(list_notifications_handler))
        .route("/notification_count", post(notification_count_handler))
        .route("/clear_notifications", post(clear_notifications_handler))
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

async fn auth_handler() -> impl IntoResponse {
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

async fn list_repos_summary_handler(State(ctx): State<Context>) -> impl IntoResponse {
    let repos = ctx.list_repos_summary()?;
    Ok::<_, Error>(Json(repos))
}

async fn seeded_not_replicated_handler(State(ctx): State<Context>) -> impl IntoResponse {
    let rids = ctx.seeded_not_replicated()?;
    Ok::<_, Error>(Json(rids))
}

async fn list_notifications_handler() -> impl IntoResponse {
    Ok::<_, Error>(Json(Vec::<
        radicle_types::domain::inbox::models::notification::NotificationsByRepo,
    >::new()))
}

async fn notification_count_handler() -> impl IntoResponse {
    Ok::<_, Error>(Json(0_usize))
}

async fn clear_notifications_handler() -> impl IntoResponse {
    Ok::<_, Error>(Json(()))
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

async fn list_repo_refs_handler(
    State(ctx): State<Context>,
    Json(RepoBody { rid }): Json<RepoBody>,
) -> impl IntoResponse {
    let refs = ctx.list_repo_refs(rid)?;

    Ok::<_, Error>(Json(refs))
}

async fn version_handler() -> impl IntoResponse {
    let version = Version {
        version: String::from("0.6.1"),
        head: String::from("51cf6cfbfe0be992ee709c49e6da589aa0f148c5"),
    };

    Ok::<_, Error>(Json(version))
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
    pub options: types::cobs::diff::DiffOptions,
}

#[derive(Serialize, Deserialize)]
struct ReadmeBody {
    pub rid: identity::RepoId,
    #[serde(default)]
    pub sha: Option<git::Oid>,
    #[serde(default)]
    pub peer: Option<NodeId>,
    #[serde(default)]
    pub revision: Option<String>,
}

async fn readme_handler(
    State(ctx): State<Context>,
    Json(ReadmeBody {
        rid,
        sha,
        peer,
        revision,
    }): Json<ReadmeBody>,
) -> impl IntoResponse {
    let readme = ctx.repo_readme(rid, sha, peer, revision)?;

    Ok::<_, Error>(Json(readme))
}

#[derive(Serialize, Deserialize)]
struct TreeBody {
    pub rid: identity::RepoId,
    pub path: PathBuf,
    #[serde(default)]
    pub sha: Option<git::Oid>,
    #[serde(default)]
    pub peer: Option<NodeId>,
    #[serde(default)]
    pub revision: Option<String>,
}

async fn tree_handler(
    State(ctx): State<Context>,
    Json(TreeBody {
        rid,
        path,
        sha,
        peer,
        revision,
    }): Json<TreeBody>,
) -> impl IntoResponse {
    let info = ctx.repo_tree(rid, path, sha, peer, revision)?;

    Ok::<_, Error>(Json(info))
}

#[derive(Serialize, Deserialize)]
struct BlobBody {
    pub rid: identity::RepoId,
    pub path: PathBuf,
    #[serde(default)]
    pub sha: Option<git::Oid>,
}

async fn blob_handler(
    State(ctx): State<Context>,
    Json(BlobBody { rid, path, sha }): Json<BlobBody>,
) -> impl IntoResponse {
    let info = ctx.repo_blob(rid, path, sha)?;

    Ok::<_, Error>(Json(info))
}

async fn diff_handler(
    State(ctx): State<Context>,
    Json(DiffBody { rid, options }): Json<DiffBody>,
) -> impl IntoResponse {
    let info = ctx.get_diff(rid, options)?;

    Ok::<_, Error>(Json(info))
}

#[derive(Serialize, Deserialize)]
struct CommitDiffBody {
    pub rid: identity::RepoId,
    pub sha: git::Oid,
    pub unified: Option<u32>,
    pub highlight: Option<bool>,
}

async fn commit_diff_handler(
    State(ctx): State<Context>,
    Json(CommitDiffBody {
        rid,
        sha,
        unified,
        highlight,
    }): Json<CommitDiffBody>,
) -> impl IntoResponse {
    let diff = ctx.get_commit_diff(rid, sha, unified, highlight)?;

    Ok::<_, Error>(Json(diff))
}

#[derive(Serialize, Deserialize)]
struct ListRepoCommitsBody {
    pub rid: identity::RepoId,
    pub head: Option<git::Oid>,
    #[serde(default)]
    pub peer: Option<NodeId>,
    #[serde(default)]
    pub revision: Option<String>,
    pub skip: Option<usize>,
    pub take: Option<usize>,
}

async fn list_repo_commits_handler(
    State(ctx): State<Context>,
    Json(ListRepoCommitsBody {
        rid,
        head,
        peer,
        revision,
        skip,
        take,
    }): Json<ListRepoCommitsBody>,
) -> impl IntoResponse {
    let commits = ctx.list_repo_commits(rid, head, peer, revision, skip, take)?;

    Ok::<_, Error>(Json(commits))
}

#[derive(Serialize, Deserialize)]
struct RepoCommitCountBody {
    pub rid: identity::RepoId,
    pub head: git::Oid,
}

async fn repo_commit_count_handler(
    State(ctx): State<Context>,
    Json(RepoCommitCountBody { rid, head }): Json<RepoCommitCountBody>,
) -> impl IntoResponse {
    let count = ctx.repo_commit_count(rid, head)?;

    Ok::<_, Error>(Json(count))
}

#[derive(Serialize, Deserialize)]
struct RepoCommitBody {
    pub rid: identity::RepoId,
    #[serde(default)]
    pub sha: Option<git::Oid>,
    #[serde(default)]
    pub peer: Option<NodeId>,
    #[serde(default)]
    pub revision: Option<String>,
}

async fn repo_commit_handler(
    State(ctx): State<Context>,
    Json(RepoCommitBody {
        rid,
        sha,
        peer,
        revision,
    }): Json<RepoCommitBody>,
) -> impl IntoResponse {
    let commit = ctx.repo_commit(rid, sha, peer, revision)?;

    Ok::<_, Error>(Json(commit))
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

async fn activity_issue_handler<
    A: serde::Serialize + serde::de::DeserializeOwned,
    B: FromRadicleAction<A> + serde::Serialize,
>(
    State(ctx): State<Context>,
    Json(ActivityBody { rid, id }): Json<ActivityBody>,
) -> impl IntoResponse {
    let activity = ctx.activity_by_id::<A, B>(rid, &radicle::cob::issue::TYPENAME, id)?;

    Ok::<_, Error>(Json(activity))
}

async fn activity_patch_handler<
    A: serde::Serialize + serde::de::DeserializeOwned,
    B: FromRadicleAction<A> + serde::Serialize,
>(
    State(ctx): State<Context>,
    Json(ActivityBody { rid, id }): Json<ActivityBody>,
) -> impl IntoResponse {
    let activity = ctx.activity_by_id::<A, B>(rid, &radicle::cob::patch::TYPENAME, id)?;

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
    pub take: Option<isize>,
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
    let aliases = profile.aliases();

    let patches = match status {
        None => ctx.patches.list(rid)?.collect::<Vec<_>>(),
        Some(s) => ctx
            .patches
            .list_by_status(rid, s.into())?
            .collect::<Vec<_>>(),
    };

    if let Some(t) = take
        && t < 0
    {
        // Return all patches
        let content = patches
            .into_iter()
            .map(|(id, patch)| models::patch::Patch::new(id, &patch, &aliases))
            .collect::<Vec<_>>();

        return Ok::<_, Error>(Json(cobs::PaginatedQuery {
            cursor: 0,
            more: false,
            content,
        }));
    }

    let take = take.unwrap_or(20) as usize;

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

#[derive(Serialize, Deserialize)]
struct JobsBody {
    pub rid: identity::RepoId,
    pub sha: git::Oid,
}

async fn jobs_handler(
    State(ctx): State<Context>,
    Json(JobsBody { rid, sha }): Json<JobsBody>,
) -> impl IntoResponse {
    let jobs = ctx.list_jobs(rid, sha)?;

    Ok::<_, Error>(Json(jobs))
}

async fn list_tags_handler(
    State(ctx): State<Context>,
    Json(RepoBody { rid }): Json<RepoBody>,
) -> impl IntoResponse {
    Ok::<_, Error>(Json(ctx.list_tags(rid)?))
}

#[derive(Serialize, Deserialize)]
struct RidBody {
    pub rid: identity::RepoId,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseByIdBody {
    pub rid: identity::RepoId,
    pub release_id: String,
}

#[derive(Serialize, Deserialize)]
struct ReleasesByCommitBody {
    pub rid: identity::RepoId,
    pub sha: git::Oid,
}

#[derive(Serialize, Deserialize)]
struct ComputeCidBody {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct CreateOrOpenReleaseBody {
    pub rid: identity::RepoId,
    pub oid: git::Oid,
    #[serde(default)]
    pub tag: Option<git::Oid>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddArtifactBody {
    pub rid: identity::RepoId,
    pub release_id: String,
    pub cid: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocationBody {
    pub rid: identity::RepoId,
    pub release_id: String,
    pub cid: String,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttestBody {
    pub rid: identity::RepoId,
    pub release_id: String,
    pub cid: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RedactBody {
    pub rid: identity::RepoId,
    pub release_id: String,
    pub cid: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetMetadataBody {
    pub rid: identity::RepoId,
    pub release_id: String,
    pub cid: String,
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveMetadataBody {
    pub rid: identity::RepoId,
    pub release_id: String,
    pub cid: String,
    pub key: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedArtifactBody {
    pub rid: identity::RepoId,
    pub release_id: String,
    pub cid: String,
    pub source_path: PathBuf,
}

async fn list_releases_handler(
    State(ctx): State<Context>,
    Json(RidBody { rid }): Json<RidBody>,
) -> impl IntoResponse {
    Ok::<_, Error>(Json(ctx.list_releases(rid)?))
}

async fn release_by_id_handler(
    State(ctx): State<Context>,
    Json(ReleaseByIdBody { rid, release_id }): Json<ReleaseByIdBody>,
) -> impl IntoResponse {
    Ok::<_, Error>(Json(ctx.release_by_id(rid, release_id)?))
}

async fn releases_by_commit_handler(
    State(ctx): State<Context>,
    Json(ReleasesByCommitBody { rid, sha }): Json<ReleasesByCommitBody>,
) -> impl IntoResponse {
    Ok::<_, Error>(Json(ctx.releases_by_commit(rid, sha)?))
}

async fn compute_artifact_cid_handler(
    State(ctx): State<Context>,
    Json(ComputeCidBody { path }): Json<ComputeCidBody>,
) -> impl IntoResponse {
    Ok::<_, Error>(Json(ctx.compute_cid(path)?))
}

async fn create_or_open_release_handler(
    State(ctx): State<Context>,
    Json(CreateOrOpenReleaseBody { rid, oid, tag }): Json<CreateOrOpenReleaseBody>,
) -> impl IntoResponse {
    Ok::<_, Error>(Json(ctx.create_or_open_release(rid, oid, tag)?))
}

async fn register_artifact_handler(
    State(ctx): State<Context>,
    Json(AddArtifactBody {
        rid,
        release_id,
        cid,
        name,
    }): Json<AddArtifactBody>,
) -> impl IntoResponse {
    ctx.register_artifact(rid, release_id, cid, name)?;
    Ok::<_, Error>(Json(()))
}

/// Stub of the native multi-file picker: write a small deterministic fixture
/// file to a temp path and return it. The frontend feeds the path into
/// `compute_artifact_cid` (which runs in this same process, so the path is
/// valid) and then `register_artifact`.
async fn pick_artifact_files_handler() -> impl IntoResponse {
    let path = std::env::temp_dir().join("radicle-desktop-e2e-artifact.txt");
    std::fs::write(&path, b"radicle-desktop release e2e fixture\n")?;
    Ok::<_, Error>(Json(vec![path.to_string_lossy().into_owned()]))
}

/// Stub of `seed_artifact`: the iroh node isn't running under the HTTP test
/// driver, so this is a no-op. The artifact is still recorded on the COB by
/// `register_artifact`; we just skip announcing an iroh location.
async fn seed_artifact_handler(Json(_): Json<SeedArtifactBody>) -> impl IntoResponse {
    Ok::<_, Error>(Json(String::new()))
}

async fn add_location_handler(
    State(ctx): State<Context>,
    Json(LocationBody {
        rid,
        release_id,
        cid,
        url,
    }): Json<LocationBody>,
) -> impl IntoResponse {
    ctx.add_location(rid, release_id, cid, url)?;
    Ok::<_, Error>(Json(()))
}

async fn remove_location_handler(
    State(ctx): State<Context>,
    Json(LocationBody {
        rid,
        release_id,
        cid,
        url,
    }): Json<LocationBody>,
) -> impl IntoResponse {
    ctx.remove_location(rid, release_id, cid, url)?;
    Ok::<_, Error>(Json(()))
}

async fn attest_artifact_handler(
    State(ctx): State<Context>,
    Json(AttestBody {
        rid,
        release_id,
        cid,
    }): Json<AttestBody>,
) -> impl IntoResponse {
    ctx.attest_artifact(rid, release_id, cid)?;
    Ok::<_, Error>(Json(()))
}

async fn set_metadata_handler(
    State(ctx): State<Context>,
    Json(SetMetadataBody {
        rid,
        release_id,
        cid,
        key,
        value,
    }): Json<SetMetadataBody>,
) -> impl IntoResponse {
    ctx.set_metadata(rid, release_id, cid, key, value)?;
    Ok::<_, Error>(Json(()))
}

async fn remove_metadata_handler(
    State(ctx): State<Context>,
    Json(RemoveMetadataBody {
        rid,
        release_id,
        cid,
        key,
    }): Json<RemoveMetadataBody>,
) -> impl IntoResponse {
    ctx.remove_metadata(rid, release_id, cid, key)?;
    Ok::<_, Error>(Json(()))
}

async fn redact_artifact_handler(
    State(ctx): State<Context>,
    Json(RedactBody {
        rid,
        release_id,
        cid,
        reason,
    }): Json<RedactBody>,
) -> impl IntoResponse {
    ctx.redact_artifact(rid, release_id, cid, reason)?;
    Ok::<_, Error>(Json(()))
}
