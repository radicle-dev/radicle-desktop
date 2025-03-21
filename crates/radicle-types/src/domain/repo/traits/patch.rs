use radicle::identity;
use radicle::patch;
use radicle::patch::Patch;
use radicle::patch::PatchId;
use radicle::patch::ReviewId;
use radicle::patch::RevisionId;

use crate::domain::repo::models;
use crate::error::Error;

pub trait RepoPatchesLister {
    fn list(
        &self,
        rid: identity::RepoId,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, models::cobs::patch::ListPatchesError>;

    fn list_by_status(
        &self,
        rid: identity::RepoId,
        status: patch::Status,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, models::cobs::patch::ListPatchesError>;
}

pub trait RepoPatches {
    fn get_patch_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
    ) -> Result<Option<models::cobs::patch::Patch>, Error>;

    fn revisions_by_patch(
        &self,
        rid: identity::RepoId,
        id: PatchId,
    ) -> Result<Option<Vec<models::cobs::patch::Revision>>, Error>;

    fn revision_by_patch_and_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: RevisionId,
    ) -> Result<Option<models::cobs::patch::Revision>, Error>;

    fn revision_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: RevisionId,
    ) -> Result<Option<models::cobs::patch::Revision>, Error>;

    fn review_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: RevisionId,
        review_id: ReviewId,
    ) -> Result<Option<models::cobs::patch::Review>, Error>;

    fn edit_patch(
        &self,
        rid: identity::RepoId,
        cob_id: PatchId,
        action: models::cobs::patch::Action,
        opts: models::cobs::CobOptions,
    ) -> Result<models::cobs::patch::Patch, Error>;
}
