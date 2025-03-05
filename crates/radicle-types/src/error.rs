use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use serde::Serialize;
use strum::{AsRefStr, EnumDiscriminants};
use ts_rs::TS;

use crate::cobs::stream;

pub mod auth;
pub mod cob;
pub mod crypto;
pub mod fs;
pub mod git;
pub mod inbox;
pub mod node;
pub mod repo;

#[derive(Debug, AsRefStr, EnumDiscriminants, thiserror::Error)]
#[strum_discriminants(derive(TS))]
#[strum_discriminants(ts(export))]
#[strum_discriminants(ts(export_to = "error/"))]
pub enum Error {
    /// Embeds error.
    #[error(transparent)]
    EmbedsError(#[from] fs::EmbedsError),

    /// Git error.
    #[error(transparent)]
    GitError(#[from] git::GitError),

    /// Node error.
    #[error(transparent)]
    NodeError(#[from] node::NodeError),

    /// Identity error.
    #[error(transparent)]
    IdentityError(#[from] auth::IdentityError),

    /// Repo error.
    #[error(transparent)]
    RepoError(#[from] repo::RepoError),

    /// Issue Cob error.
    #[error(transparent)]
    IssueError(#[from] cob::IssueError),

    /// Patch Cob error.
    #[error(transparent)]
    PatchError(#[from] cob::PatchError),

    /// Issue Cob error.
    #[error(transparent)]
    CryptoError(#[from] crypto::CryptoError),

    /// Inbox error.
    #[error(transparent)]
    InboxError(#[from] inbox::InboxError),

    /// Profile error.
    #[error(transparent)]
    ProfileError(#[from] radicle::profile::Error),

    /// Config error.
    #[error(transparent)]
    ConfigError(#[from] radicle::profile::ConfigError),

    /// CobStore error.
    #[error(transparent)]
    CobStore(#[from] radicle::cob::store::Error),

    /// Io error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Sqlite error.
    #[error(transparent)]
    Sqlite(#[from] sqlite::Error),

    /// Io error.
    #[error(transparent)]
    Actions(#[from] stream::error::Actions),

    /// CobStream error.
    #[error(transparent)]
    CobStream(#[from] stream::error::Stream),

    /// Node database error.
    #[error(transparent)]
    Database(#[from] radicle::node::db::Error),

    /// Repository error.
    #[error(transparent)]
    Repository(#[from] radicle::storage::RepositoryError),

    /// Storage error.
    #[error(transparent)]
    Storage(#[from] radicle::storage::Error),

    /// Serde JSON error.
    #[error(transparent)]
    SerdeJSON(#[from] serde_json::error::Error),
}

#[derive(Serialize, TS, Debug)]
pub struct ErrorWrapper {
    r#type: String,
    message: String,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let error_type = match self {
            Error::NodeError(error) => format!("{}.{}", self.as_ref(), error.as_ref()),
            Error::IdentityError(error) => format!("{}.{}", self.as_ref(), error.as_ref()),
            Error::GitError(error) => format!("{}.{}", self.as_ref(), error.as_ref()),
            Error::EmbedsError(error) => format!("{}.{}", self.as_ref(), error.as_ref()),
            Error::RepoError(error) => format!("{}.{}", self.as_ref(), error.as_ref()),
            Error::IssueError(error) => format!("{}.{}", self.as_ref(), error.as_ref()),
            Error::PatchError(error) => format!("{}.{}", self.as_ref(), error.as_ref()),
            Error::InboxError(error) => format!("{}.{}", self.as_ref(), error.as_ref()),
            _ => self.as_ref().to_string(),
        };

        ErrorWrapper {
            r#type: error_type,
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::to_string(&self).unwrap(),
        )
            .into_response()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use crate::error::{auth::IdentityError, Error};

    #[test]
    fn serialize_nested_errors() {
        let serialized =
            serde_json::to_string(&Error::IdentityError(IdentityError::InvalidPassphrase)).unwrap();
        assert_eq!(
            serialized,
            "{\"type\":\"IdentityError.InvalidPassphrase\",\"message\":\"invalid passphrase\"}"
        );
    }
}
