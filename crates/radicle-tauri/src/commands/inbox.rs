use std::collections::BTreeMap;

use radicle::{git, identity, Profile};

use radicle_types::domain::inbox::models::notification;
use radicle_types::domain::inbox::service::Service;
use radicle_types::domain::inbox::traits::InboxService;
use radicle_types::domain::repo::models::cobs;
use radicle_types::error::Error;
use radicle_types::outbound::sqlite::Sqlite;

#[tauri::command]
pub fn list_notifications(
    profile: tauri::State<Profile>,
    service: tauri::State<Service<Sqlite>>,
    params: notification::RepoGroupParams,
) -> Result<
    cobs::PaginatedQuery<BTreeMap<git::Qualified<'static>, Vec<notification::NotificationItem>>>,
    Error,
> {
    service.list_notifications(&profile, params)
}

#[tauri::command]
pub fn count_notifications_by_repo(
    profile: tauri::State<Profile>,
    service: tauri::State<Service<Sqlite>>,
) -> Result<BTreeMap<identity::RepoId, notification::NotificationCount>, Error> {
    service.count_notifications_by_repo(profile.storage.clone())
}

#[tauri::command]
pub fn clear_notifications(
    profile: tauri::State<Profile>,
    service: tauri::State<Service<Sqlite>>,
    params: notification::SetStatusNotifications,
) -> Result<(), Error> {
    service.clear_notifications(&profile, params)
}
