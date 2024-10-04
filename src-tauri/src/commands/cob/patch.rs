use radicle::git;
use radicle::identity::RepoId;
use radicle::patch::cache::Patches;
use radicle::storage::ReadStorage;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Error;
use crate::types::cobs;
use crate::AppState;

use crate::cob::query;

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PaginatedQuery<T> {
    pub cursor: usize,
    pub more: bool,
    pub content: T,
}

#[tauri::command]
pub async fn list_patches(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    status: Option<query::PatchStatus>,
    skip: Option<usize>,
    take: Option<usize>,
) -> Result<PaginatedQuery<Vec<cobs::Patch>>, Error> {
    let cursor = skip.unwrap_or(0);
    let take = take.unwrap_or(20);
    let repo = ctx.profile.storage.repository(rid)?;
    let aliases = &ctx.profile.aliases();
    let cache = ctx.profile.patches(&repo)?;
    let patches = match status {
        None => cache.list()?.collect::<Vec<_>>(),
        Some(s) => cache.list_by_status(&s.into())?.collect::<Vec<_>>(),
    };
    let more = cursor + take < patches.len();

    let mut patches = patches
        .into_iter()
        .filter_map(|p| {
            p.map(|(id, patch)| cobs::Patch::new(id, patch, aliases))
                .ok()
        })
        .skip(cursor)
        .take(take)
        .collect::<Vec<_>>();

    patches.sort_by_key(|b| std::cmp::Reverse(b.timestamp()));

    Ok::<_, Error>(PaginatedQuery {
        cursor,
        more,
        content: patches,
    })
}

#[tauri::command]
pub fn patch_by_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    id: git::Oid,
) -> Result<Option<cobs::Patch>, Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let patches = ctx.profile.patches(&repo)?;
    let patch = patches.get(&id.into())?;
    let aliases = &ctx.profile.aliases();
    let patches = patch.map(|patch| cobs::Patch::new(id.into(), patch, aliases));

    Ok::<_, Error>(patches)
}

#[tauri::command]
pub fn revisions_by_patch(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    id: git::Oid,
) -> Result<Option<Vec<cobs::Revision>>, Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let patches = ctx.profile.patches(&repo)?;
    let revisions = patches.get(&id.into())?.map(|patch| {
        let aliases = &ctx.profile.aliases();

        patch
            .revisions()
            .map(|(_, r)| cobs::Revision::new(r.clone(), aliases))
            .collect::<Vec<_>>()
    });

    Ok::<_, Error>(revisions)
}

#[tauri::command]
pub fn revision_by_patch_and_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    id: git::Oid,
    revision_id: git::Oid,
) -> Result<Option<cobs::Revision>, Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let patches = ctx.profile.patches(&repo)?;
    let revision = patches.get(&id.into())?.and_then(|patch| {
        let aliases = &ctx.profile.aliases();

        patch
            .revision(&revision_id.into())
            .map(|r| cobs::Revision::new(r.clone(), aliases))
    });
    Ok::<_, Error>(revision)
}
