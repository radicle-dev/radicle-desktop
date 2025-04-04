use radicle::git;

use crate::domain::inbox::models::notification::{
    CountByRepo, ListNotificationsError, NotificationRow, RepoGroupParams,
};
use crate::domain::inbox::traits::{InboxService, InboxStorage};

use super::models::notification::SetStatusNotifications;

#[derive(Debug, Clone)]
pub struct Service<I>
where
    I: InboxStorage,
{
    inbox: I,
}

impl<I> Service<I>
where
    I: InboxStorage,
{
    pub fn new(inbox: I) -> Self {
        Self { inbox }
    }
}

impl<I> InboxService for Service<I>
where
    I: InboxStorage,
{
    fn counts_by_repo(
        &self,
    ) -> Result<
        impl Iterator<Item = Result<CountByRepo, ListNotificationsError>>,
        ListNotificationsError,
    > {
        self.inbox.counts_by_repo()
    }

    fn repo_group(
        &self,
        params: RepoGroupParams,
    ) -> Result<
        std::collections::BTreeMap<git::Qualified<'static>, Vec<NotificationRow>>,
        ListNotificationsError,
    > {
        self.inbox.repo_group(params)
    }

    fn list_notifications(
        &self,
        profile: &radicle::Profile,
        params: RepoGroupParams,
    ) -> Result<
        crate::domain::repo::models::cobs::PaginatedQuery<
            std::collections::BTreeMap<
                git::Qualified<'static>,
                Vec<super::models::notification::NotificationItem>,
            >,
        >,
        crate::error::Error,
    > {
        self.inbox.list_notifications(profile, params)
    }

    fn count_notifications_by_repo(
        &self,
        storage: &radicle::Storage,
    ) -> Result<
        std::collections::BTreeMap<
            radicle::identity::RepoId,
            super::models::notification::NotificationCount,
        >,
        crate::error::Error,
    > {
        self.inbox.count_notifications_by_repo(storage)
    }

    fn clear_notifications(
        &self,
        profile: &radicle::Profile,
        params: SetStatusNotifications,
    ) -> Result<(), crate::error::Error> {
        self.inbox.clear_notifications(profile, params)
    }
}
