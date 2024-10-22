use serde_json::json;

use radicle::crypto::Verified;
use radicle::identity::doc::PayloadId;
use radicle::identity::{DocAt, RepoId};
use radicle::issue::cache::Issues;
use radicle::node::routing::Store;
use radicle::patch::cache::Patches;
use radicle::prelude::Doc;
use radicle::storage::git::Repository;
use radicle::storage::ReadStorage;
use radicle::storage::{self, ReadRepository};
use radicle_types as types;

use crate::error::Error;
use crate::AppState;

/// List all repos.
#[tauri::command]
pub fn list_repos(ctx: tauri::State<AppState>) -> Result<Vec<types::repo::RepoInfo>, Error> {
    let storage = &ctx.profile.storage;
    let policies = ctx.profile.policies()?;
    let mut repos = storage.repositories()?.into_iter().collect::<Vec<_>>();
    repos.sort_by_key(|p| p.rid);

    let infos = repos
        .into_iter()
        .filter_map(|info| {
            if !policies.is_seeding(&info.rid).unwrap_or_default() {
                return None;
            }
            let repo = ctx.profile.storage.repository(info.rid).ok()?;
            let DocAt { doc, .. } = repo.identity_doc().ok()?;
            let repo_info = repo_info(&ctx.profile, &repo, &doc).ok()?;

            Some(repo_info)
        })
        .collect::<Vec<_>>();

    Ok::<_, Error>(infos)
}

#[tauri::command]
pub fn repo_by_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
) -> Result<types::repo::RepoInfo, Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let DocAt { doc, .. } = repo.identity_doc()?;

    let repo_info = repo_info(&ctx.profile, &repo, &doc)?;

    Ok::<_, Error>(repo_info)
}

#[tauri::command]
pub async fn diff_stats(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: String,
    head: String,
) -> Result<types::cobs::Stats, Error> {
    let repo = radicle_surf::Repository::open(storage::git::paths::repository(
        &ctx.profile.storage,
        &rid,
    ))?;
    let base = repo.commit(base)?;
    let commit = repo.commit(head)?;
    let diff = repo.diff(base.id, commit.id)?;
    let stats = diff.stats();

    Ok::<_, Error>(types::cobs::Stats::new(stats))
}

pub fn repo_info(
    profile: &radicle::Profile,
    repo: &Repository,
    doc: &Doc<Verified>,
) -> Result<types::repo::RepoInfo, Error> {
    let aliases = profile.aliases();
    let delegates = doc
        .delegates
        .clone()
        .into_iter()
        .map(|did| types::cobs::Author::new(did, &aliases))
        .collect::<Vec<_>>();
    let db = profile.database()?;
    let seeding = db.count(&repo.id).unwrap_or_default();
    let project = doc.payload.get(&PayloadId::project()).and_then(|payload| {
        let (_, head) = repo.head().ok()?;
        let commit = repo.commit(head).ok()?;
        let patches = profile.patches(repo).ok()?;
        let patches = patches.counts().ok()?;
        let issues = profile.issues(repo).ok()?;
        let issues = issues.counts().ok()?;

        Some(json!({
            "data": payload,
            "meta": {
                "issues": issues,
                "patches": patches,
                "head": head,
                "lastCommit": commit.time().seconds() * 1000,
            },
        }))
    });

    Ok::<_, Error>(types::repo::RepoInfo {
        payloads: types::repo::SupportedPayloads { project },
        delegates,
        threshold: doc.threshold,
        visibility: doc.visibility.clone(),
        rid: repo.id,
        seeding,
    })
}
