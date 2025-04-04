use std::collections::BTreeMap;

use radicle::{git, identity};

use crate::domain::inbox::models::notification::{
    CountByRepo, ListNotificationsError, NotificationCount, NotificationItem, RepoGroup,
    RepoGroupParams,
};
use crate::domain::repo::models::cobs::PaginatedQuery;
use crate::error::Error;

use super::models::notification::SetStatusNotifications;

pub trait InboxStorage {
    fn clear_notifications(
        &self,
        profile: &radicle::Profile,
        params: SetStatusNotifications,
    ) -> Result<(), Error>;

    fn count_notifications_by_repo(
        &self,
        storage: &radicle::Storage,
    ) -> Result<BTreeMap<identity::RepoId, NotificationCount>, Error>;

    fn list_notifications(
        &self,
        profile: &radicle::Profile,
        params: RepoGroupParams,
    ) -> Result<PaginatedQuery<BTreeMap<git::Qualified<'static>, Vec<NotificationItem>>>, Error>;

    fn counts_by_repo(
        &self,
    ) -> Result<
        impl Iterator<Item = Result<CountByRepo, ListNotificationsError>>,
        ListNotificationsError,
    >;

    fn repo_group(&self, params: RepoGroupParams) -> Result<RepoGroup, ListNotificationsError>;
}

pub trait InboxService {
    fn clear_notifications(
        &self,
        profile: &radicle::Profile,
        params: SetStatusNotifications,
    ) -> Result<(), Error>;

    fn count_notifications_by_repo(
        &self,
        storage: &radicle::Storage,
    ) -> Result<BTreeMap<identity::RepoId, NotificationCount>, Error>;

    fn list_notifications(
        &self,
        profile: &radicle::Profile,
        params: RepoGroupParams,
    ) -> Result<PaginatedQuery<BTreeMap<git::Qualified<'static>, Vec<NotificationItem>>>, Error>;

    /// Get the total notification count by repos.
    fn counts_by_repo(
        &self,
    ) -> Result<
        impl Iterator<Item = Result<CountByRepo, ListNotificationsError>>,
        ListNotificationsError,
    >;

    fn repo_group(&self, params: RepoGroupParams) -> Result<RepoGroup, ListNotificationsError>;
}
