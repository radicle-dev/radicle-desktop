use std::str::FromStr;

use radicle::cob::ObjectId;
use radicle::git::Oid;
use radicle::identity::RepoId;
use radicle::patch::cache::Patches;

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
    let (repo, _) = ctx.repo(rid)?;
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
    id: String,
) -> Result<Option<cobs::Patch>, Error> {
    let id = ObjectId::from_str(&id)?;
    let (repo, _) = ctx.repo(rid)?;
    let patches = ctx.profile.patches(&repo)?;
    let patch = patches.get(&id)?;

    let aliases = &ctx.profile.aliases();
    let patches = patch.map(|patch| cobs::Patch::new(id, patch, aliases));

    Ok::<_, Error>(patches)
}

#[tauri::command]
pub fn revisions_by_patch(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    id: String,
) -> Result<Option<Vec<cobs::Revision>>, Error> {
    let id = ObjectId::from_str(&id)?;
    let (repo, _) = ctx.repo(rid)?;
    let patches = ctx.profile.patches(&repo)?;

    let revisions = patches.get(&id)?.map(|patch| {
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
    id: String,
    revision_id: String,
) -> Result<Option<cobs::Revision>, Error> {
    let id = ObjectId::from_str(&id)?;
    let (repo, _) = ctx.repo(rid)?;
    let patches = ctx.profile.patches(&repo)?;
    let revision = patches.get(&id)?.and_then(|patch| {
        let revision_id = Oid::from_str(&revision_id).ok()?;
        let aliases = &ctx.profile.aliases();

        patch
            .revision(&revision_id.into())
            .map(|r| cobs::Revision::new(r.clone(), aliases))
    });
    Ok::<_, Error>(revision)
}
