use radicle::cob;
use radicle::git;
use radicle::identity;

use radicle_types::traits::cobs::Cobs;

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
    ctx.publish_draft(rid, cob_id, type_name)
        .map_err(Error::from)
}
