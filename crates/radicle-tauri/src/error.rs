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

    /// Cob patch cache error.
    #[error(transparent)]
    CachePatch(#[from] radicle::cob::patch::cache::Error),

    /// Cob issue cache error.
    #[error(transparent)]
    CacheIssue(#[from] radicle::cob::issue::cache::Error),

    /// Repository error.
    #[error(transparent)]
    Repository(#[from] radicle::storage::RepositoryError),

    /// Crypto error.
    #[error(transparent)]
    Crypto(#[from] radicle::crypto::ssh::keystore::Error),

    /// SSH Agent error.
    #[error(transparent)]
    Agent(#[from] radicle::crypto::ssh::agent::Error),

    /// Radicle Git error.
    #[error(transparent)]
    Git(#[from] radicle::git::Error),

    /// Git2 error.
    #[error(transparent)]
    Git2(#[from] radicle::git::raw::Error),

    /// Diff error.
    #[error(transparent)]
    Diff(#[from] radicle_surf::diff::git::error::Diff),

    /// Storage error.
    #[error(transparent)]
    Storage(#[from] radicle::storage::Error),

    /// Surf error.
    #[error(transparent)]
    Surf(#[from] radicle_surf::Error),

    /// Issue error.
    #[error(transparent)]
    Issue(#[from] radicle::issue::Error),

    /// Patch error.
    #[error(transparent)]
    Patch(#[from] radicle::patch::Error),

    /// An error with a hint.
    #[error("{err} {hint}")]
    WithHint {
        err: anyhow::Error,
        hint: &'static str,
    },

    /// Node error.
    #[error(transparent)]
    Node(#[from] radicle::node::Error),
}

#[derive(Serialize)]
struct ErrorWrapperWithHint {
    err: String,
    hint: String,
}

#[derive(Serialize)]
struct ErrorWrapper {
    err: String,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        if let Error::WithHint { err, hint } = self {
            let error_wrapper = ErrorWrapperWithHint {
                err: err.to_string(),
                hint: hint.to_string(),
            };

            return error_wrapper.serialize(serializer);
        }

        let wrapper = ErrorWrapper {
            err: self.to_string(),
        };

        wrapper.serialize(serializer)
    }
}
