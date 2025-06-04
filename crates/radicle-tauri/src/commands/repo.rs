use std::collections::BTreeSet;
use std::str::FromStr;

use radicle::git;
use radicle::identity::project::ProjectName;
use radicle::identity::{doc, Project, RepoId, Visibility};
use radicle::rad::InitError;
use radicle::storage::git::Repository;
use radicle::storage::refs::branch_of;
use radicle::storage::{SignRepository, WriteRepository};
use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::repo::{Repo, Show};

use crate::AppState;

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
pub fn repo_readme(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    sha: Option<git::Oid>,
) -> Result<Option<types::repo::Readme>, Error> {
    ctx.repo_readme(rid, sha)
}

#[tauri::command]
pub async fn diff_stats(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    base: git::Oid,
    head: git::Oid,
) -> Result<types::diff::Stats, Error> {
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
    name: String,
    description: String,
) -> Result<(), Error> {
    let profile = &ctx.profile;
    let storage = &profile.storage;
    let signer = ctx.profile.signer()?;
    let config = radicle::git::raw::Config::open_default()?;
    // SAFETY: "master" is always a valid RefString
    let default_branch = git::RefString::try_from(
        config
            .get_string("init.defaultBranch")
            .unwrap_or("master".to_owned()),
    )
    .unwrap();

    let name = ProjectName::from_str(&name)?;
    if description.len() > doc::MAX_STRING_LENGTH {
        return Err(Error::ProjectError(
            radicle::identity::project::ProjectError::Description("Cannot exceed 255 characters."),
        ));
    }

    let visibility = Visibility::Private {
        allow: BTreeSet::default(),
    };

    let proj = Project::new(name, description, default_branch.clone()).map_err(|errs| {
        InitError::ProjectPayload(
            errs.into_iter()
                .map(|err| err.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        )
    })?;
    let doc = radicle::identity::Doc::initial(proj, profile.public_key.into(), visibility);
    let (project, identity) = Repository::init(&doc, &storage, &signer)?;

    let tree_id = {
        let mut index = project.backend.index()?;

        index.write_tree()
    }?;
    let sig = project.backend.signature()?;
    let tree = project.backend.find_tree(tree_id)?;

    project.set_remote_identity_root_to(signer.public_key(), identity)?;
    project.set_identity_head_to(identity)?;

    let base = project
        .backend
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    let ns_head = branch_of(&ctx.profile.public_key, &default_branch);
    project
        .backend
        .reference(ns_head.as_str(), base, false, "Created namespace ref")?;

    project.set_head()?;
    project.sign_refs(&signer)?;

    Ok(())
}

#[tauri::command]
pub fn seed(ctx: tauri::State<AppState>, rid: RepoId) -> Result<(), Error> {
    ctx.seed(rid)
}

#[tauri::command]
pub fn unseed(ctx: tauri::State<AppState>, rid: RepoId) -> Result<(), Error> {
    ctx.unseed(rid)
}

#[tauri::command]
pub fn seeded_not_replicated(ctx: tauri::State<AppState>) -> Result<Vec<RepoId>, Error> {
    ctx.seeded_not_replicated()
}
