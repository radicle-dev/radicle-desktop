use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time;

use radicle::git::Qualified;
use radicle::{git, identity};
use sqlite as sql;

use crate::domain::inbox::models::notification;
use crate::domain::inbox::traits::InboxStorage;
use crate::error::Error;

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

impl InboxStorage for Sqlite {
    fn counts_by_repo(
        &self,
    ) -> Result<
        impl Iterator<Item = Result<notification::CountByRepo, notification::ListNotificationsError>>,
        notification::ListNotificationsError,
    > {
        let stmt = self.db.prepare(
            "SELECT repo, COUNT(*) as count
                 FROM `repository-notifications`
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
                BTreeMap<Qualified<'static>, Vec<notification::NotificationRow>>,
                notification::ListNotificationsError,
            >>()
    }
}
