use strum::{AsRefStr, EnumDiscriminants};
use ts_rs::TS;

#[derive(Debug, AsRefStr, EnumDiscriminants, thiserror::Error)]
#[strum_discriminants(derive(TS))]
#[strum_discriminants(ts(export))]
#[strum_discriminants(ts(export_to = "error/"))]
pub enum IssueError {
    /// Cob issue cache error.
    #[error(transparent)]
    CacheIssue(#[from] radicle::cob::issue::cache::Error),

    /// Issue error.
    #[error(transparent)]
    Issue(#[from] radicle::issue::Error),
}

#[derive(Debug, AsRefStr, EnumDiscriminants, thiserror::Error)]
#[strum_discriminants(derive(TS))]
#[strum_discriminants(ts(export))]
#[strum_discriminants(ts(export_to = "error/"))]
pub enum PatchError {
    /// Cob patch cache error.
    #[error(transparent)]
    CachePatch(#[from] radicle::cob::patch::cache::Error),

    /// Patch error.
    #[error(transparent)]
    Patch(#[from] radicle::patch::Error),

    /// CobStore error.
    #[error(transparent)]
    ListPatchesError(#[from] crate::domain::patch::models::patch::ListPatchesError),
}
