use std::collections::BTreeSet;

use radicle::identity::project::ProjectName;
use radicle::identity::{Project, RepoId, Visibility};
use radicle::node::{policy, Handle};
use radicle::prelude::Did;
use radicle::rad::InitError;
use radicle::storage::git::Repository;
use radicle::storage::refs::branch_of;
use radicle::storage::{SignRepository, WriteRepository};
use radicle::{git, Node};
use radicle_types as types;
use radicle_types::error::node::NodeError;
use radicle_types::error::Error;
use radicle_types::traits::repo::{Repo, Show};

use crate::AppState;

use super::node::NODE_FETCH_TIMEOUT;

#[tauri::command]
pub fn list_repos(
    ctx: tauri::State<AppState>,
    show: Show,
) -> Result<Vec<types::repo::RepoInfo>, Error> {
    ctx.list_repos(show)
}

#[tauri::command]
pub fn repo_count(ctx: tauri::State<AppState>) -> Result<types::repo::RepoCount, Error> {
    ctx.repo_count()
}

#[tauri::command]
pub fn repo_by_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
) -> Result<types::repo::RepoInfo, Error> {
    ctx.repo_by_id(rid)
}

#[tauri::command]
pub async fn diff_stats(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: git::Oid,
    head: git::Oid,
) -> Result<types::cobs::Stats, Error> {
    ctx.diff_stats(rid, base, head)
}

#[tauri::command]
pub async fn list_commits(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: String,
    head: String,
) -> Result<Vec<types::repo::Commit>, Error> {
    ctx.list_commits(rid, base, head)
}

#[tauri::command]
pub(crate) async fn create_repo(
    ctx: tauri::State<'_, AppState>,
    name: ProjectName,
    description: String,
    default_branch: git::RefString,
) -> Result<(), Error> {
    let profile = &ctx.profile;
    let storage = &profile.storage;
    let signer = ctx.profile.signer()?;

    let visibility = Visibility::Private {
        allow: BTreeSet::default(),
    };

    let proj = Project::new(name, description, default_branch.clone())
        .map_err(|errs| {
            InitError::ProjectPayload(
                errs.into_iter()
                    .map(|err| err.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        })
        .map_err(types::error::repo::RepoError::ProjectInitError)?;
    let doc = radicle::identity::Doc::initial(proj, profile.public_key.into(), visibility);
    let (project, identity) = Repository::init(&doc, &storage, &signer)?;

    let tree_id = {
        let mut index = project
            .backend
            .index()
            .map_err(types::error::git::GitError::Git2)?;

        index
            .write_tree()
            .map_err(types::error::git::GitError::Git2)
    }?;
    let sig = project
        .backend
        .signature()
        .map_err(types::error::git::GitError::Git2)?;
    let tree = project
        .backend
        .find_tree(tree_id)
        .map_err(types::error::git::GitError::Git2)?;

    project.set_remote_identity_root_to(signer.public_key(), identity)?;
    project.set_identity_head_to(identity)?;

    let base = project
        .backend
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .map_err(types::error::git::GitError::Git2)?;

    let ns_head = branch_of(&ctx.profile.public_key, &default_branch);
    project
        .backend
        .reference(ns_head.as_str(), base, false, "Created namespace ref")
        .map_err(types::error::git::GitError::Git2)?;

    project.set_head()?;
    project.sign_refs(&signer)?;

    Ok(())
}

#[tauri::command]
pub(crate) async fn fetch_repo(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    from: Did,
) -> Result<(), Error> {
    let profile = &ctx.profile;
    let mut node = Node::new(profile.socket());

    if !node.is_running() {
        return Err(NodeError::NotRunningError.into());
    }

    node.add_inventory(rid)
        .map_err(types::error::node::NodeError::Node)?;
    node.seed(rid, policy::Scope::All)
        .map_err(types::error::node::NodeError::Node)?;
    if !node
        .fetch(rid, from.into(), NODE_FETCH_TIMEOUT)
        .map_err(types::error::node::NodeError::Node)?
        .is_success()
    {
        return Err(NodeError::RepoFetchError.into());
    }

    Ok(())
}
