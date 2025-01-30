use radicle::identity;
use radicle::patch;
use radicle::patch::Patch;
use radicle::patch::PatchId;

use crate::domain::patch::models;

pub trait PatchStorage {
    fn list(
        &self,
        rid: identity::RepoId,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, models::patch::ListPatchesError>;

    fn list_by_status(
        &self,
        rid: identity::RepoId,
        status: patch::Status,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, models::patch::ListPatchesError>;
}

pub trait PatchService {
    fn list(
        &self,
        rid: identity::RepoId,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, models::patch::ListPatchesError>;

    fn list_by_status(
        &self,
        rid: identity::RepoId,
        status: patch::Status,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, models::patch::ListPatchesError>;
}
