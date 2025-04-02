use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use serde::Serialize;

use crate::cobs::stream;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Profile error.
    #[error(transparent)]
    ProfileError(#[from] radicle::profile::Error),

    /// Missing SSH Agent error.
    #[error("ssh agent not running")]
    AgentNotRunning,

    /// Embeds error.
    #[error("not able to save embed")]
    SaveEmbedError,

    /// Init Error error.
    #[error(transparent)]
    InitError(#[from] radicle::rad::InitError),

    /// Alias error.
    #[error(transparent)]
    AliasError(#[from] radicle::node::AliasError),

    /// Tauri Plugin Clipboard error.
    #[error(transparent)]
    TauriPluginClipboard(#[from] tauri_plugin_clipboard_manager::Error),

    /// Tauri Plugin Fs error.
    #[error(transparent)]
    TauriPluginFs(#[from] tauri_plugin_fs::Error),

    /// Project error.
    #[error(transparent)]
    ProjectError(#[from] radicle::identity::project::ProjectError),

    /// List notification error.
    #[error(transparent)]
    ListNotificationsError(
        #[from] crate::domain::inbox::models::notification::ListNotificationsError,
    ),

    /// CobStore error.
    #[error(transparent)]
    ListPatchesError(#[from] crate::domain::patch::models::patch::ListPatchesError),

    #[error(transparent)]
    PatchCountsError(#[from] crate::domain::patch::models::patch::CountsError),

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

    /// Inbox error.
    #[error(transparent)]
    Inbox(#[from] radicle::node::notifications::Error),

    /// Crypto error.
    #[error(transparent)]
    Crypto(#[from] radicle::crypto::ssh::keystore::Error),

    /// SSH Agent error.
    #[error(transparent)]
    Agent(#[from] radicle::crypto::ssh::agent::Error),

    /// Node database error.
    #[error(transparent)]
    Database(#[from] radicle::node::db::Error),

    /// Repository error.
    #[error(transparent)]
    Repository(#[from] radicle::storage::RepositoryError),

    /// Policy store error.
    #[error(transparent)]
    PolicyStore(#[from] radicle::node::policy::store::Error),

    /// Cob patch cache error.
    #[error(transparent)]
    CachePatch(#[from] radicle::cob::patch::cache::Error),

    /// Diff error.
    #[error(transparent)]
    Diff(#[from] radicle_surf::diff::git::error::Diff),

    /// Storage error.
    #[error(transparent)]
    Storage(#[from] radicle::storage::Error),

    /// Radicle Git error.
    #[error(transparent)]
    Git(#[from] radicle::git::Error),

    /// Surf error.
    #[error(transparent)]
    Surf(#[from] radicle_surf::Error),

    /// Git2 error.
    #[error(transparent)]
    Git2(#[from] radicle::git::raw::Error),

    /// Cob issue cache error.
    #[error(transparent)]
    CacheIssue(#[from] radicle::cob::issue::cache::Error),

    /// Patch error.
    #[error(transparent)]
    Patch(#[from] radicle::patch::Error),

    /// Issue error.
    #[error(transparent)]
    Issue(#[from] radicle::issue::Error),

    /// Node error.
    #[error(transparent)]
    Node(#[from] radicle::node::Error),

    /// Serde JSON error.
    #[error(transparent)]
    SerdeJSON(#[from] serde_json::error::Error),
}

impl Error {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Error::ProjectError(radicle::identity::project::ProjectError::Name(_)) => {
                "ProjectError.InvalidName"
            }
            Error::ProjectError(radicle::identity::project::ProjectError::Description(_)) => {
                "ProjectError.InvalidDescription"
            }
            Error::Crypto(radicle::crypto::ssh::keystore::Error::Ssh(ssh_key::Error::Crypto))
            | Error::Crypto(radicle::crypto::ssh::keystore::Error::PassphraseMissing) => {
                "PassphraseError.InvalidPassphrase"
            }
            Error::AliasError(radicle::node::AliasError::Empty) => "AliasError.EmptyAlias",
            Error::AliasError(radicle::node::AliasError::MaxBytesExceeded) => {
                "AliasError.TooLongAlias"
            }
            Error::ProfileError(radicle::profile::Error::NotFound(_)) => {
                "IdentityError.MissingProfile"
            }
            Error::AliasError(radicle::node::AliasError::InvalidCharacter) => {
                "AliasError.InvalidAlias"
            }
            _ => "UnknownError",
        }
    }
}

#[derive(Serialize, ts_rs::TS, Debug)]
#[ts(export)]
#[ts(export_to = "error/")]
pub struct ErrorWrapper {
    code: String,
    #[ts(optional)]
    message: Option<String>,
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("ErrorWrapper", 2)?;
        state.serialize_field("code", &self.code().to_string())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
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
    use crate::error::Error;

    #[test]
    fn serialize_nested_errors() {
        let serialized = serde_json::to_string(&Error::Crypto(
            radicle::crypto::ssh::keystore::Error::Ssh(ssh_key::Error::Crypto),
        ))
        .unwrap();
        assert_eq!(
            serialized,
            "{\"code\":\"PassphraseError.InvalidPassphrase\",\"message\":\"ssh keygen: cryptographic error\"}"
        );
    }

    #[test]
    fn serialize_unknown_errors() {
        let serialized =
            serde_json::to_string(&Error::Issue(radicle::issue::Error::MissingIdentity)).unwrap();
        assert_eq!(
            serialized,
            "{\"code\":\"UnknownError\",\"message\":\"identity document missing\"}"
        );
    }
}
