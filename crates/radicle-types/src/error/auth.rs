use radicle::crypto::ssh::keystore::MemorySignerError;
use strum::{AsRefStr, EnumDiscriminants};
use ts_rs::TS;

#[derive(Debug, AsRefStr, EnumDiscriminants, thiserror::Error)]
#[strum_discriminants(derive(TS))]
#[strum_discriminants(ts(export))]
#[strum_discriminants(ts(export_to = "error/"))]
pub enum IdentityError {
    #[error("passphrase can't be empty.")]
    MissingPassphrase,

    /// Alias error.
    #[error(transparent)]
    AliasError(#[from] radicle_node::node::AliasError),

    #[error(transparent)]
    MemorySignerError(#[from] MemorySignerError),

    #[error("ssh agent not running")]
    SSHAgentNotRunning,

    #[error("invalid passphrase")]
    InvalidPassphrase,

    #[error("key not found")]
    KeyNotFoundError,

    /// Crypto error.
    #[error(transparent)]
    Crypto(#[from] radicle::crypto::ssh::keystore::Error),
}
