use radicle::{cob, git, identity, patch};

use radicle_types::domain::repo::models::cobs;
use radicle_types::domain::repo::service::Service;
use radicle_types::domain::repo::traits::RepoService as _;
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle;
use radicle_types::outbound::sqlite::Sqlite;

#[tauri::command]
pub async fn list_patches(
    profile: tauri::State<'_, radicle::Profile>,
    service: tauri::State<'_, Service<Radicle, Sqlite>>,
    rid: identity::RepoId,
    status: Option<cobs::query::PatchStatus>,
    skip: Option<usize>,
    // None: return all patches, `skip` is ignored.
    take: Option<usize>,
) -> Result<cobs::PaginatedQuery<Vec<cobs::patch::Patch>>, Error> {
    let aliases = &profile.aliases();
    let patches = match status {
        Some(status) => service
            .list_patches_by_status(rid, status.into())?
            .collect::<Vec<_>>(),
        None => service.list_patches(rid)?.collect::<Vec<_>>(),
    };

    match take {
        None => {
            let content = patches
                .into_iter()
                .map(|(id, patch)| cobs::patch::Patch::new(&id, &patch, aliases))
                .collect::<Vec<_>>();

            Ok::<_, Error>(cobs::PaginatedQuery {
                cursor: 0,
                more: false,
                content,
            })
        }
        Some(take) => {
            let total_count = patches.len();
            let cursor = skip.unwrap_or(0);

            Ok(
                cobs::PaginatedQuery::<Vec<cobs::patch::Patch>>::map_with_pagination(
                    patches.into_iter(),
                    total_count,
                    cursor,
                    take,
                    |(id, patch)| cobs::patch::Patch::new(&id, &patch, aliases),
                ),
            )
        }
    }
}

#[tauri::command]
pub fn patch_by_id(
    repo_service: tauri::State<Service<Radicle, Sqlite>>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Option<cobs::patch::Patch>, Error> {
    repo_service.get_patch_by_id(rid, id.into())
}

#[tauri::command]
pub fn revisions_by_patch(
    repo_service: tauri::State<Service<Radicle, Sqlite>>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Option<Vec<cobs::patch::Revision>>, Error> {
    repo_service.revisions_by_patch(rid, id.into())
}

#[tauri::command]
pub fn revision_by_patch_and_id(
    repo_service: tauri::State<Service<Radicle, Sqlite>>,
    rid: identity::RepoId,
    id: git::Oid,
    revision_id: git::Oid,
) -> Result<Option<cobs::patch::Revision>, Error> {
    repo_service.revision_by_id(rid, id.into(), revision_id.into())
}

#[tauri::command]
pub fn review_by_patch_and_revision_and_id(
    repo_service: tauri::State<Service<Radicle, Sqlite>>,
    rid: identity::RepoId,
    id: git::Oid,
    revision_id: git::Oid,
    review_id: cob::patch::ReviewId,
) -> Result<Option<cobs::patch::Review>, Error> {
    repo_service.review_by_id(rid, id.into(), revision_id.into(), review_id)
}

#[tauri::command]
pub fn edit_patch(
    repo_service: tauri::State<Service<Radicle, Sqlite>>,
    rid: identity::RepoId,
    cob_id: git::Oid,
    action: cobs::patch::Action,
    opts: cobs::CobOptions,
) -> Result<cobs::patch::Patch, Error> {
    repo_service.edit_patch(rid, cob_id.into(), action, opts)
}

#[tauri::command]
pub fn activity_by_patch(
    service: tauri::State<Service<Radicle, Sqlite>>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Vec<cobs::Operation<cobs::patch::Action>>, Error> {
    service.activity_by_id(rid, &patch::TYPENAME, id)
}
