use radicle_types::domain::inbox::models::notification::{RepoGroupParams, SetStatusNotifications};

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ListNotificationsPayload {
    pub params: RepoGroupParams,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ClearNotificationsPayload {
    pub params: SetStatusNotifications,
}
