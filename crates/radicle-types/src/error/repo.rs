use radicle::identity::PayloadError;
use strum::{AsRefStr, EnumDiscriminants};
use ts_rs::TS;

#[derive(Debug, AsRefStr, EnumDiscriminants, thiserror::Error)]
#[strum_discriminants(derive(TS))]
#[strum_discriminants(ts(export))]
#[strum_discriminants(ts(export_to = "error/"))]
pub enum RepoError {
    /// Payload error.
    #[error(transparent)]
    PayloadError(#[from] PayloadError),

    /// Initialize Project error.
    #[error(transparent)]
    ProjectInitError(#[from] radicle::rad::InitError),
}
