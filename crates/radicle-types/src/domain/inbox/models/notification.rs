use radicle::{git, identity, node, storage};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename = "camelCase")]
pub struct NotificationRow {
    pub row_id: node::notifications::NotificationId,
    pub timestamp: localtime::LocalTime,
    pub remote: Option<storage::RemoteId>,
    pub old: Option<git::Oid>,
    pub new: Option<git::Oid>,
}

pub type RepoGroup = std::collections::BTreeMap<git::Qualified<'static>, Vec<NotificationRow>>;
pub type CountByRepo = (identity::RepoId, usize);

#[derive(Clone, Debug)]
pub struct CountsByRepoParams {
    pub repo: identity::RepoId,
}

#[derive(Clone, Debug)]
pub struct RepoGroupParams {
    pub repo: identity::RepoId,
}

#[derive(Debug, thiserror::Error)]
pub enum ListNotificationsError {
    #[error(transparent)]
    RefError(#[from] git::RefError),

    #[error(transparent)]
    SerdeJSON(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlite(#[from] sqlite::Error),

    #[error(transparent)]
    NotificationKindError(#[from] node::notifications::NotificationKindError),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    // to be extended as new error scenarios are introduced
}
