use radicle::cob;
use radicle::cob::object::Storage;
use radicle::git;
use radicle::git::refs::storage::draft;
use radicle::identity;
use radicle::storage;
use radicle::storage::ReadStorage;

use crate::error::Error;
use crate::AppState;

/// Puts a draft of a Collaborative Object (COB) out of the draft state by updating the reference to the new object ID (OID).
///
/// The function updates the reference for the provided `type_name` (e.g., patch, issue) to point to the
/// new object ID (OID) associated with the finalized draft, removing it from the draft store.
#[tauri::command]
pub fn publish_draft(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    cob_id: git::Oid,
    type_name: cob::TypeName,
) -> Result<(), Error> {
    let signer = ctx.profile.signer()?;
    let repo = ctx.profile.storage.repository(rid)?;
    let draft_oid =
        repo.backend
            .refname_to_id(&draft::cob(signer.public_key(), &type_name, &cob_id.into()))?;
    repo.update(
        signer.public_key(),
        &type_name,
        &cob_id.into(),
        &draft_oid.into(),
    )?;

    let mut patches = ctx.profile.patches_mut(&repo)?;
    patches.write(&cob_id.into())?;

    storage::git::cob::DraftStore::new(&repo, *signer.public_key()).remove(
        signer.public_key(),
        &type_name,
        &cob_id.into(),
    )?;

    Ok::<_, Error>(())
}
