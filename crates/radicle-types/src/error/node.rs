use strum::{AsRefStr, EnumDiscriminants};
use ts_rs::TS;

#[derive(Debug, AsRefStr, EnumDiscriminants, thiserror::Error)]
#[strum_discriminants(derive(TS))]
#[strum_discriminants(ts(export))]
#[strum_discriminants(ts(export_to = "error/"))]
pub enum NodeError {
    #[error("node start error")]
    StartError,

    #[error("node connect error")]
    ConnectError,

    #[error("node running error")]
    NotRunningError,

    #[error("node fetch error")]
    RepoFetchError,

    /// Config error.
    #[error(transparent)]
    ConfigError(#[from] radicle::profile::config::ConfigError),

    /// Policy store error.
    #[error(transparent)]
    PolicyStore(#[from] radicle::node::policy::store::Error),

    /// Runtime error.
    #[error(transparent)]
    RuntimeError(#[from] radicle_node::runtime::Error),

    /// Node error.
    #[error(transparent)]
    Node(#[from] radicle::node::Error),
}
