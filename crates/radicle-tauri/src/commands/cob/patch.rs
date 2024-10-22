use serde::{Deserialize, Serialize};
use ts_rs::TS;

use radicle::cob;
use radicle::git;
use radicle::identity;
use radicle::patch;
use radicle::patch::cache::Patches;
use radicle::storage;
use radicle::storage::ReadStorage;
use radicle_types::cobs;
use radicle_types::thread;

use crate::cob::query;
use crate::error::Error;
use crate::AppState;

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
    rid: identity::RepoId,
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
    rid: identity::RepoId,
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
    rid: identity::RepoId,
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
    rid: identity::RepoId,
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

/// Creates a draft review for a specific patch revision.
///
/// This Tauri command allows users to create a new draft review for a specific patch revision.
/// The draft is associated with the user (signer) and the provided patch revision within the repository.
#[tauri::command]
pub fn create_draft_review(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    revision_id: cob::patch::RevisionId,
    cob_id: git::Oid,
    labels: Vec<cob::Label>,
) -> Result<patch::ReviewId, Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let signer = ctx.profile.signer()?;
    let drafts = storage::git::cob::DraftStore::new(&repo, *signer.public_key());

    let mut patches = cob::patch::Cache::no_cache(&drafts)?;
    let mut patch = patches.get_mut(&cob_id.into())?;
    let revision = patch
        .revision(&revision_id)
        .ok_or_else(|| Error::WithHint {
            err: anyhow::anyhow!("patch revision not found"),
            hint: "Not able to find the specified patch revision.",
        })?;

    revision
        .review_by(signer.public_key())
        .ok_or(Error::WithHint {
            err: anyhow::anyhow!("duplicate patch review found"),
            hint: "Found an existing draft patch review on this patch revision and repo.",
        })?;

    let review_id = patch.review(
        revision.id(),
        Some(cob::patch::Verdict::Reject),
        None,
        labels,
        &signer,
    )?;

    patches.write(&cob_id.into())?;

    Ok::<_, Error>(review_id)
}

/// Creates a new review comment on a draft review for a specific patch.
///
/// This Tauri command is used to add a comment to an existing draft review in a repository.
/// It allows users to comment on a specific location in the code or leave general feedback
/// on a review that belongs to a specific patch.
#[tauri::command]
pub fn create_draft_review_comment(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    cob_id: git::Oid,
    new: thread::CreateReviewComment,
) -> Result<(), Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let signer = ctx.profile.signer()?;
    let drafts = storage::git::cob::DraftStore::new(&repo, *signer.public_key());

    let mut patches = cob::patch::Cache::no_cache(&drafts)?;
    let mut patch = patches.get_mut(&cob_id.into())?;

    patch.transaction("Review comments", &signer, |tx| {
        tx.review_comment(
            new.review_id,
            new.body,
            new.location.map(|l| l.into()),
            new.reply_to,
            new.embeds,
        )?;

        Ok(())
    })?;

    patches.write(&cob_id.into())?;

    Ok::<_, Error>(())
}

/// Edits a draft review for a specific patch revision in a repository.
///
/// This Tauri command allows users to edit a draft review for a specific patch review.
/// The draft is associated with the user (signer) and the provided patch revision within the repository.
#[tauri::command]
pub fn edit_draft_review(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    cob_id: git::Oid,
    edit: cobs::ReviewEdit,
) -> Result<(), Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let signer = ctx.profile.signer()?;
    let drafts = storage::git::cob::DraftStore::new(&repo, *signer.public_key());

    let mut patches = cob::patch::Cache::no_cache(&drafts)?;
    let mut patch = patches.get_mut(&cob_id.into())?;
    patch.review_edit(
        edit.review_id,
        edit.verdict,
        edit.summary,
        edit.labels,
        &signer,
    )?;

    patches.write(&cob_id.into())?;

    Ok::<_, Error>(())
}

/// Gets the draft review of the local user for a specific patch revision in a repository.
///
/// This Tauri command is used to retrieve a patch review draft for the local user
/// on a given patch revision from a repository.
/// It looks up the repository using the provided repository ID (`rid`) and patch object ID (`cob_id`),
/// and gets the patch review of the local user associated with a specific revision (`revision_id`), if it exists.
#[tauri::command]
pub fn get_draft_review(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    cob_id: git::Oid,
    revision_id: patch::RevisionId,
) -> Option<patch::Review> {
    let repo = ctx.profile.storage.repository(rid).ok()?;
    let signer = ctx.profile.signer().ok()?;
    let drafts = storage::git::cob::DraftStore::new(&repo, *signer.public_key());
    let patches = cob::patch::Cache::no_cache(&drafts).ok()?;

    let patch = patches.get(&cob_id.into()).ok()?;
    let revision = patch.and_then(|p| p.revision(&revision_id).cloned());
    let review = revision.and_then(|rev| rev.review_by(signer.public_key()).cloned());

    review
}
