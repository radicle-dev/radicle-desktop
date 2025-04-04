use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time;

use radicle::issue::cache::Issues as _;
use radicle::patch::cache::Patches as _;
use radicle::patch::{Patch, PatchId, Status};
use radicle::storage::{ReadRepository, ReadStorage};
use radicle::{git, identity};
use sqlite as sql;

use crate::domain::inbox::models::notification;
use crate::domain::inbox::traits::InboxStorage;
use crate::domain::repo::models::cobs::patch::ListPatchesError;
use crate::domain::repo::models::cobs::PaginatedQuery;
use crate::domain::repo::traits::patch::RepoPatchesLister;
use crate::error::Error;

#[derive(Clone)]
pub struct Sqlite {
    pub db: Arc<sql::ConnectionThreadSafe>,
}

impl Sqlite {
    /// How long to wait for the database lock to be released before failing a read.
    const DB_READ_TIMEOUT: time::Duration = time::Duration::from_secs(3);

    pub fn reader<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut db = sql::Connection::open_thread_safe_with_flags(
            path,
            sqlite::OpenFlags::new().with_read_only(),
        )?;
        db.set_busy_timeout(Self::DB_READ_TIMEOUT.as_millis() as usize)?;

        Ok(Self { db: Arc::new(db) })
    }
}

impl RepoPatchesLister for Sqlite {
    fn list(
        &self,
        rid: identity::RepoId,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, ListPatchesError> {
        let mut stmt = self.db.prepare(
            "SELECT id, patch, (
                 SELECT MIN(JSON_EXTRACT(revision.value, '$.timestamp'))
                 FROM JSON_EACH(JSON_EXTRACT(p.patch, '$.revisions')) AS revision
             ) AS last_revision_timestamp
             FROM patches AS p
             WHERE repo = ?1
             ORDER BY last_revision_timestamp DESC;
             ",
        )?;
        stmt.bind((1, &rid))?;
        Ok(stmt.into_iter().filter_map(|row| {
            let row = row.ok()?;
            let id = PatchId::from_str(row.read::<&str, _>("id")).ok()?;
            let patch = serde_json::from_str::<Patch>(row.read::<&str, _>("patch")).ok()?;
            Some((id, patch))
        }))
    }

    fn list_by_status(
        &self,
        rid: identity::RepoId,
        status: Status,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, ListPatchesError> {
        let mut stmt = self.db.prepare(
            "SELECT id, patch, (
                 SELECT MIN(JSON_EXTRACT(revision.value, '$.timestamp'))
                 FROM JSON_EACH(JSON_EXTRACT(p.patch, '$.revisions')) AS revision
             ) AS last_revision_timestamp
             FROM patches AS p
             WHERE repo = ?1
             AND patch->>'$.state.status' = ?2
             ORDER BY last_revision_timestamp DESC;
             ",
        )?;
        stmt.bind((1, &rid))?;
        stmt.bind((2, sql::Value::String(status.to_string())))?;
        Ok(stmt.into_iter().filter_map(|row| {
            let row = row.ok()?;
            let id = PatchId::from_str(row.read::<&str, _>("id")).ok()?;
            let patch = serde_json::from_str::<Patch>(row.read::<&str, _>("patch")).ok()?;
            Some((id, patch))
        }))
    }
}

impl InboxStorage for Sqlite {
    fn count_notifications_by_repo(
        &self,
        storage: &radicle::Storage,
    ) -> Result<BTreeMap<identity::RepoId, notification::NotificationCount>, Error> {
        let result = self
            .counts_by_repo()?
            .filter_map(|s| {
                let (rid, count) = s.ok()?;
                let repo = storage.repository(rid).ok()?;
                let identity::DocAt { doc, .. } = repo.identity_doc().ok()?;
                let project = doc.project().ok()?;

                Some((
                    rid,
                    notification::NotificationCount {
                        rid,
                        name: project.name().to_string(),
                        count,
                    },
                ))
            })
            .collect::<BTreeMap<identity::RepoId, notification::NotificationCount>>();

        Ok(result)
    }
    fn list_notifications(
        &self,
        profile: &radicle::Profile,
        params: notification::RepoGroupParams,
    ) -> Result<
        PaginatedQuery<BTreeMap<git::Qualified<'static>, Vec<notification::NotificationItem>>>,
        Error,
    > {
        let aliases = profile.aliases();
        let cursor = params.skip.unwrap_or(0);
        let take = params.take.unwrap_or(20);

        let all = self.repo_group(params.clone())?;
        let more = cursor + take < all.len();
        let repo = profile.storage.repository(params.repo)?;
        let patches = profile.patches(&repo)?;
        let issues = profile.issues(&repo)?;

        let content = all
            .into_iter()
            .skip(cursor)
            .take(take)
            .map(|(qualified, n)| {
                let items = n
                    .into_iter()
                    .filter_map(|s| {
                        let update: notification::RefUpdate =
                            (qualified.clone().into_refstring(), s.new, s.old).into();
                        let update: radicle::storage::RefUpdate = update.into();
                        let kind = radicle::node::notifications::NotificationKind::try_from(
                            qualified.clone(),
                        )
                        .ok()?;

                        match kind {
                            radicle::node::notifications::NotificationKind::Cob {
                                ref typed_id,
                            } => {
                                if typed_id.is_patch() {
                                    let actions = notification::actions(
                                        typed_id.type_name.clone(),
                                        typed_id.id,
                                        update.old(),
                                        update.new(),
                                        &repo,
                                        &aliases,
                                    )
                                    .unwrap_or_default();

                                    match patches.get(&typed_id.id) {
                                        Ok(Some(p)) => Some(notification::NotificationItem::Patch(
                                            notification::Patch {
                                                row_id: s.row_id,
                                                id: typed_id.id,
                                                update: update.into(),
                                                timestamp: s.timestamp,
                                                title: p.title().to_string(),
                                                status: (p.state().clone()).into(),
                                                actions,
                                            },
                                        )),
                                        Ok(None) => {
                                            log::error!("No patch found");
                                            None
                                        }
                                        Err(e) => {
                                            log::error!("{}", e);
                                            None
                                        }
                                    }
                                } else if typed_id.is_issue() {
                                    let actions = notification::actions(
                                        typed_id.type_name.clone(),
                                        typed_id.id,
                                        update.old(),
                                        update.new(),
                                        &repo,
                                        &aliases,
                                    )
                                    .unwrap_or_default();

                                    match issues.get(&typed_id.id) {
                                        Ok(Some(i)) => Some(notification::NotificationItem::Issue(
                                            notification::Issue {
                                                row_id: s.row_id,
                                                id: typed_id.id,
                                                update: update.into(),
                                                timestamp: s.timestamp,
                                                title: i.title().to_string(),
                                                status: (*i.state()).into(),
                                                actions,
                                            },
                                        )),
                                        Ok(None) => {
                                            log::error!("No issue found");
                                            None
                                        }
                                        Err(e) => {
                                            log::error!("{}", e);
                                            None
                                        }
                                    }
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();

                (qualified, items)
            })
            .filter(|(_, v)| !v.is_empty())
            .collect::<BTreeMap<git::Qualified<'static>, Vec<notification::NotificationItem>>>();

        Ok(PaginatedQuery {
            cursor,
            more,
            content,
        })
    }

    fn counts_by_repo(
        &self,
    ) -> Result<
        impl Iterator<Item = Result<notification::CountByRepo, notification::ListNotificationsError>>,
        notification::ListNotificationsError,
    > {
        let stmt = self.db.prepare(
            "SELECT ref, repo, COUNT(*) as count
                 FROM `repository-notifications`
                 WHERE ref LIKE '%cobs%'
                 GROUP BY repo",
        )?;

        Ok(stmt.into_iter().map(|row| {
            let row = row?;
            let count = row.try_read::<i64, _>("count")? as usize;
            let repo = row.try_read::<identity::RepoId, _>("repo")?;

            Ok((repo, count))
        }))
    }

    fn repo_group(
        &self,
        params: notification::RepoGroupParams,
    ) -> Result<
        std::collections::BTreeMap<git::Qualified<'static>, Vec<notification::NotificationRow>>,
        notification::ListNotificationsError,
    > {
        let mut stmt = self.db.prepare(
        "SELECT ref, substr(ref, 66) ref_without_namespace, json_group_array(json_object('row_id', rowid, 'timestamp', timestamp, 'remote', substr(ref, 17, 48), 'old', old, 'new', new)) as value
        FROM 'repository-notifications'
        WHERE repo = ?
        GROUP BY ref_without_namespace
        ORDER BY timestamp DESC"
    )?;
        stmt.bind((1, &params.repo))?;

        stmt.into_iter()
            .map(|row| {
                let row = row?;
                let refstr = row.try_read::<&str, _>("ref")?;
                let value = row.try_read::<&str, _>("value")?;
                let items = serde_json::from_str::<Vec<notification::NotificationRow>>(value)?;
                let (_, reference) = git::parse_ref::<String>(refstr)?;

                Ok((reference.to_owned(), items))
            })
            .collect::<Result<
                BTreeMap<git::Qualified<'static>, Vec<notification::NotificationRow>>,
                notification::ListNotificationsError,
            >>()
    }

    fn clear_notifications(
        &self,
        profile: &radicle::Profile,
        params: notification::SetStatusNotifications,
    ) -> Result<(), Error> {
        let mut notifications = profile.notifications_mut()?;
        match params {
            notification::SetStatusNotifications::Ids(ids) => notifications.clear(&ids)?,
            notification::SetStatusNotifications::Repo(repo) => {
                notifications.clear_by_repo(&repo)?
            }
            notification::SetStatusNotifications::All => notifications.clear_all()?,
        };

        Ok(())
    }
}
