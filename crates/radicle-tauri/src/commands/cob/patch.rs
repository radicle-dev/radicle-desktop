use radicle::patch::TYPENAME;
use radicle::{cob, git, identity};

use radicle_types as types;
use radicle_types::cobs;
use radicle_types::domain::patch::models;
use radicle_types::domain::patch::service::Service;
use radicle_types::domain::patch::traits::PatchService;
use radicle_types::error::Error;
use radicle_types::outbound::sqlite::Sqlite;
use radicle_types::traits::cobs::Cobs;
use radicle_types::traits::patch::Patches;
use radicle_types::traits::patch::PatchesMut;
use radicle_types::traits::Profile;

use crate::AppState;

#[tauri::command]
pub async fn list_patches(
    ctx: tauri::State<'_, AppState>,
    sqlite_service: tauri::State<'_, Service<Sqlite>>,
    rid: identity::RepoId,
    status: Option<types::cobs::query::PatchStatus>,
    skip: Option<usize>,
    take: Option<usize>,
) -> Result<types::cobs::PaginatedQuery<Vec<models::patch::Patch>>, Error> {
    let profile = ctx.profile();
    let cursor = skip.unwrap_or(0);
    let take = take.unwrap_or(20);
    let aliases = profile.aliases();
    let patches = match status {
        None => sqlite_service.list(rid)?.collect::<Vec<_>>(),
        Some(s) => sqlite_service
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

    Ok::<_, Error>(cobs::PaginatedQuery {
        cursor,
        more,
        content: patches,
    })
}

#[tauri::command]
pub fn patch_by_id(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Option<models::patch::Patch>, Error> {
    ctx.get_patch(rid, id)
}

#[tauri::command]
pub fn revisions_by_patch(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Option<Vec<models::patch::Revision>>, Error> {
    ctx.revisions_by_patch(rid, id)
}

#[tauri::command]
pub fn revision_by_patch_and_id(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    id: git::Oid,
    revision_id: git::Oid,
) -> Result<Option<models::patch::Revision>, Error> {
    ctx.revision_by_id(rid, id, revision_id)
}

#[tauri::command]
pub fn review_by_patch_and_revision_and_id(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    id: git::Oid,
    revision_id: git::Oid,
    review_id: cob::patch::ReviewId,
) -> Result<Option<models::patch::Review>, Error> {
    ctx.review_by_id(rid, id, revision_id, review_id)
}

#[tauri::command]
pub fn edit_patch(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    cob_id: git::Oid,
    action: models::patch::Action,
    opts: cobs::CobOptions,
) -> Result<models::patch::Patch, Error> {
    ctx.edit_patch(rid, cob_id, action, opts)
}

#[tauri::command]
pub fn activity_by_patch(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Vec<types::cobs::Operation<models::patch::Action>>, Error> {
    ctx.activity_by_id(rid, &TYPENAME, id)
}
