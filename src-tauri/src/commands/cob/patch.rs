use radicle::git;
use radicle::identity::RepoId;
use radicle::patch::cache::Patches;
use radicle::storage::ReadStorage;

use crate::cob::query;
use crate::error::Error;
use crate::types::cobs;
use crate::AppState;

#[tauri::command]
pub fn list_patches(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    status: query::PatchStatus,
) -> Result<Vec<cobs::Patch>, Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let patches = ctx.profile.patches(&repo)?;
    let mut patches: Vec<_> = patches
        .list()?
        .filter_map(|r| {
            let (id, patch) = r.ok()?;
            (status.matches(patch.state())).then_some((id, patch))
        })
        .collect::<Vec<_>>();

    patches.sort_by(|(_, a), (_, b)| b.timestamp().cmp(&a.timestamp()));
    let aliases = &ctx.profile.aliases();
    let patches = patches
        .into_iter()
        .map(|(id, patch)| cobs::Patch::new(id, patch, aliases))
        .collect::<Vec<_>>();

    Ok::<_, Error>(patches)
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
