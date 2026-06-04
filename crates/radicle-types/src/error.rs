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

    /// Profile signer error.
    #[error(transparent)]
    Signer(#[from] radicle::profile::SignerError),

    /// Radicle error.
    #[error("radicle is not installed")]
    RadicleNotInstalled,

    /// Missing SSH Agent error.
    #[error("ssh agent not running")]
    AgentNotRunning,

    /// File size too big
    #[error("file size too large: {0}")]
    FileTooLarge(usize),

    /// Generic Cob cache error.
    #[error(transparent)]
    Cache(#[from] radicle::cob::cache::Error),

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

    /// Tauri error.
    #[error(transparent)]
    Tauri(#[from] tauri::Error),

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
    Agent(#[from] radicle::crypto::ssh::agent::AgentError),

    /// SSH Agent connection error.
    #[error(transparent)]
    AgentConnect(#[from] radicle::crypto::ssh::agent::ConnectError),

    /// Node database error.
    #[error(transparent)]
    Database(#[from] radicle::node::db::Error),

    /// Repository error.
    #[error(transparent)]
    SurfFsError(#[from] radicle_surf::fs::error::Directory),

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

    /// Storage refs error.
    #[error(transparent)]
    StorageRefs(#[from] radicle::storage::refs::Error),

    /// Revision not found.
    #[error("revision not found: {0}")]
    RevisionNotFound(String),

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

    /// Invalid title.
    #[error(transparent)]
    Title(#[from] radicle::cob::TitleError),

    /// Node error.
    #[error(transparent)]
    Node(#[from] radicle::node::Error),

    /// Serde JSON error.
    #[error(transparent)]
    SerdeJSON(#[from] serde_json::error::Error),

    /// Iroh / iroh-blobs error.
    #[error("iroh: {0}")]
    Iroh(String),

    /// Release creation error.
    #[error(transparent)]
    ReleaseCreate(#[from] radicle_artifact::error::Create),

    /// Release redaction error.
    #[error(transparent)]
    ReleaseRedact(#[from] radicle_artifact::error::Redact),

    /// Release metadata validation error.
    #[error(transparent)]
    ReleaseMetadata(#[from] radicle_artifact::error::Metadata),

    /// Artifact share error (CID parsing, hashing, content addressing).
    #[error(transparent)]
    ArtifactShare(#[from] radicle_artifact_core::Error),

    /// CID parse error.
    #[error(transparent)]
    Cid(#[from] cid::Error),

    /// URL parse error.
    #[error(transparent)]
    Url(#[from] url::ParseError),

    /// COB object id parse error.
    #[error(transparent)]
    ParseObjectId(#[from] radicle::cob::object::ParseObjectId),

    /// File picker / dialog closed before returning a result.
    #[error("dialog was closed before returning a result")]
    DialogClosed,

    /// No iroh locations reachable for the requested CID.
    #[error("no reachable iroh location for {cid}")]
    NoReachableLocations { cid: String },

    /// The artifact has no usable locations of any supported scheme.
    #[error("no locations available for {cid}")]
    NoLocations { cid: String },

    /// All transport attempts (iroh + HTTP) failed for the requested CID.
    /// The aggregated messages from each attempt are joined into one string
    /// for surfacing in the UI.
    #[error("all transports failed for {cid}: {reasons}")]
    AllTransportsFailed { cid: String, reasons: String },

    /// Release with the given id was not found in the COB store.
    #[error("release {release_id} not found")]
    ReleaseNotFound { release_id: String },

    /// Artifact CID is not registered against the given release.
    #[error("artifact {cid} not in release {release_id}")]
    ArtifactNotInRelease { cid: String, release_id: String },

    /// Persisted iroh secret key file does not contain 32 bytes.
    #[error("malformed iroh key at {path}")]
    MalformedIrohKey { path: std::path::PathBuf },

    /// CID computed from imported content does not match the expected CID.
    #[error("cid mismatch: expected {expected}, got {actual}")]
    CidMismatch { expected: String, actual: String },

    /// The external `rad-artifact` node is not reachable on its control
    /// socket. Seeding and downloading need it; the rest of the app does
    /// not, so this is surfaced as a recoverable, action-specific error.
    #[error("rad-artifact node is not running")]
    ArtifactNodeNotRunning,

    /// Error talking to the `rad-artifact` node over its control socket.
    #[error(transparent)]
    ArtifactClient(radicle_artifact_client::ClientError),
}

impl From<radicle_artifact_client::ClientError> for Error {
    fn from(e: radicle_artifact_client::ClientError) -> Self {
        use radicle_artifact_client::ClientError;
        // A missing or refused socket means the node isn't up; map it to
        // a dedicated variant the frontend can detect to prompt setup.
        if let ClientError::Io(io) = &e
            && matches!(
                io.kind(),
                std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
            )
        {
            return Error::ArtifactNodeNotRunning;
        }
        Error::ArtifactClient(e)
    }
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
            Error::FileTooLarge(_) => "PayloadError.TooLarge",
            Error::DialogClosed => "DialogError.Closed",
            Error::NoReachableLocations { .. } => "ArtifactError.NoReachableLocations",
            Error::NoLocations { .. } => "ArtifactError.NoLocations",
            Error::AllTransportsFailed { .. } => "ArtifactError.AllTransportsFailed",
            Error::ReleaseNotFound { .. } => "ReleaseError.NotFound",
            Error::ArtifactNotInRelease { .. } => "ReleaseError.ArtifactNotFound",
            Error::CidMismatch { .. } => "ArtifactError.CidMismatch",
            Error::ArtifactNodeNotRunning => "ArtifactError.NodeNotRunning",
            Error::MalformedIrohKey { .. } => "IrohError.MalformedKey",
            Error::ReleaseRedact(radicle_artifact::error::Redact::NotFound { .. }) => {
                "ReleaseError.ArtifactNotFound"
            }
            Error::ReleaseRedact(radicle_artifact::error::Redact::ReasonTooLong { .. }) => {
                "ReleaseError.RedactionReasonTooLong"
            }
            Error::ReleaseCreate(radicle_artifact::error::Create::MissingTag { .. }) => {
                "ReleaseError.MissingTag"
            }
            Error::ReleaseCreate(radicle_artifact::error::Create::TagMismatch { .. }) => {
                "ReleaseError.TagMismatch"
            }
            Error::ReleaseCreate(radicle_artifact::error::Create::PeelFailed { .. }) => {
                "ReleaseError.TagPeelFailed"
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
