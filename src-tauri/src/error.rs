use serde::Serialize;

/// Errors relating to the API backend.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Profile error.
    #[error(transparent)]
    Profile(#[from] radicle::profile::Error),

    /// Node database error.
    #[error(transparent)]
    Database(#[from] radicle::node::db::Error),

    /// Policy store error.
    #[error(transparent)]
    PolicyStore(#[from] radicle::node::policy::store::Error),

    #[error(transparent)]
    CobsCache(#[from] radicle::cob::cache::Error),

    /// Cob patch cache error.
    #[error(transparent)]
    CachePatch(#[from] radicle::cob::patch::cache::Error),

    /// Cob issue cache error.
    #[error(transparent)]
    CacheIssue(#[from] radicle::cob::issue::cache::Error),

    /// Repository error.
    #[error(transparent)]
    Repository(#[from] radicle::storage::RepositoryError),

    /// Project doc error.
    #[error(transparent)]
    ProjectDoc(#[from] radicle::identity::doc::PayloadError),

    /// Notification store error.
    #[error(transparent)]
    NotificationsStore(#[from] radicle::node::notifications::store::Error),

    /// Routing error.
    #[error(transparent)]
    Routing(#[from] radicle::node::routing::Error),

    /// Git2 error.
    #[error(transparent)]
    Git2(#[from] radicle::git::raw::Error),

    /// Storage refs error.
    #[error(transparent)]
    StorageRef(#[from] radicle::storage::refs::Error),

    /// Surf error.
    #[error(transparent)]
    Surf(#[from] radicle_surf::Error),

    /// Storage error.
    #[error(transparent)]
    Storage(#[from] radicle::storage::Error),

    /// Node error.
    #[error(transparent)]
    Node(#[from] radicle::node::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
