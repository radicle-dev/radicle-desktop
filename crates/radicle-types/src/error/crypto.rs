use strum::{AsRefStr, EnumDiscriminants};
use ts_rs::TS;

#[derive(Debug, AsRefStr, EnumDiscriminants, thiserror::Error)]
#[strum_discriminants(derive(TS))]
#[strum_discriminants(ts(export))]
#[strum_discriminants(ts(export_to = "error/"))]
pub enum CryptoError {
    /// Crypto error.
    #[error(transparent)]
    Crypto(#[from] radicle::crypto::ssh::keystore::Error),

    /// SSH Agent error.
    #[error(transparent)]
    Agent(#[from] radicle::crypto::ssh::agent::Error),
}
