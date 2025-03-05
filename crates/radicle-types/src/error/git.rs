use strum::{AsRefStr, EnumDiscriminants};
use ts_rs::TS;

#[derive(Debug, AsRefStr, EnumDiscriminants, thiserror::Error)]
#[strum_discriminants(derive(TS))]
#[strum_discriminants(ts(export))]
#[strum_discriminants(ts(export_to = "error/"))]
pub enum GitError {
    /// Radicle Git error.
    #[error(transparent)]
    Git(#[from] radicle::git::Error),

    /// GitRefFormat error.
    #[error(transparent)]
    GitRefFormat(#[from] radicle::git::fmt::Error),

    /// Git2 error.
    #[error(transparent)]
    Git2(#[from] radicle::git::raw::Error),

    /// Diff error.
    #[error(transparent)]
    Diff(#[from] radicle_surf::diff::git::error::Diff),

    /// Surf error.
    #[error(transparent)]
    Surf(#[from] radicle_surf::Error),
}
