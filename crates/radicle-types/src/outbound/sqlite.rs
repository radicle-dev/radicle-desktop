use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time;

use radicle::patch::{Patch, PatchId, Status};
use radicle::{git, identity};
use sqlite as sql;

use crate::domain::inbox::models::notification;
use crate::domain::inbox::traits::InboxStorage;
use crate::domain::patch::models::patch::{CountsError, ListPatchesError, PatchCounts, State};
use crate::domain::patch::traits::PatchStorage;
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

impl PatchStorage for Sqlite {
    fn counts(&self, rid: identity::RepoId) -> Result<PatchCounts, CountsError> {
        let mut stmt = self.db.prepare(
            "SELECT
                 patch->'$.state' AS state,
                 COUNT(*) AS count
             FROM patches
             WHERE repo = ?1
             GROUP BY patch->'$.state.status'",
        )?;
        stmt.bind((1, &rid))?;

        stmt.into_iter()
            .try_fold(PatchCounts::default(), |mut counts, row| {
                let row = row?;
                let count = row.read::<i64, _>("count") as usize;
                let status = serde_json::from_str::<State>(row.read::<&str, _>("state"))
                    .map_err(|err| CountsError::Unknown(err.into()))?;
                match status {
                    State::Draft => counts.draft += count,
                    State::Open { .. } => counts.open += count,
                    State::Archived => counts.archived += count,
                    State::Merged { .. } => counts.merged += count,
                }
                Ok(counts)
            })
    }

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
    fn counts_by_repo(
        &self,
    ) -> Result<
        impl Iterator<Item = Result<notification::CountByRepo, notification::ListNotificationsError>>,
        notification::ListNotificationsError,
    > {
        let stmt = self.db.prepare(
            "SELECT COUNT(DISTINCT substr(ref, 66)) count, ref, repo
                 FROM `repository-notifications`
                 WHERE new NOT NULL AND (ref LIKE '%cobs/xyz.radicle.patch%' OR ref LIKE '%cobs/xyz.radicle.issue%')
                 GROUP BY repo",
        )?;

        Ok(stmt.into_iter().map(|row| {
            let row = row?;
            let count = row.try_read::<i64, _>("count")? as usize;
            let repo = row.try_read::<identity::RepoId, _>("repo")?;

            Ok((repo, count))
        }))
    }

    fn notification_count(&self) -> Result<usize, notification::ListNotificationsError> {
        let stmt = self.db.prepare(
            "SELECT COUNT(DISTINCT substr(ref, 66)) as count
             FROM `repository-notifications`
             WHERE new NOT NULL AND (ref LIKE '%cobs/xyz.radicle.patch%' OR ref LIKE '%cobs/xyz.radicle.issue%')",
        )?;

        match stmt.into_iter().next() {
            Some(Ok(row)) => Ok(row.try_read::<i64, _>("count")? as usize),
            _ => Ok(0),
        }
    }

    fn repo_group(
        &self,
        params: notification::RepoGroupParams,
    ) -> Result<
        Vec<(identity::RepoId, notification::RepoGroup)>,
        notification::ListNotificationsError,
    > {
        let repos_clause = match &params.repos {
            Some(repos) if !repos.is_empty() => {
                let placeholders: Vec<String> =
                    (1..=repos.len()).map(|i| format!("?{}", i)).collect();
                format!("WHERE repo IN ({})", placeholders.join(","))
            }
            _ => String::from(""),
        };

        let query = format!(
            "SELECT repo, ref, substr(ref, 66) ref_without_namespace,
                json_group_array(
                    json_object(
                        'row_id', rowid,
                        'timestamp', timestamp,
                        'remote', substr(ref, 17, 48),
                        'old', old,
                        'new', new
                    )
                ) as value,
                MAX(timestamp) AS latest_timestamp
            FROM 'repository-notifications'
            {}
            GROUP BY repo, ref_without_namespace
            ORDER BY latest_timestamp DESC",
            repos_clause
        );

        let mut stmt = self.db.prepare(&query)?;

        if let Some(repos) = &params.repos {
            if !repos.is_empty() {
                for (i, repo) in repos.iter().enumerate() {
                    stmt.bind((i + 1, repo))?;
                }
            }
        }

        let mut result: Vec<(identity::RepoId, notification::RepoGroup)> = Vec::new();
        let mut current_repo: Option<identity::RepoId> = None;
        let mut current_group: notification::RepoGroup = Vec::new();

        for row_result in stmt.into_iter() {
            let row = row_result?;
            let repo_id = row.try_read::<identity::RepoId, _>("repo")?;
            let refstr = row.try_read::<&str, _>("ref")?;
            let value = row.try_read::<&str, _>("value")?;
            let items = serde_json::from_str::<Vec<notification::NotificationRow>>(value)?;
            let (_, reference) = git::parse_ref::<String>(refstr)?;

            if let Some(current) = current_repo {
                if current != repo_id {
                    result.push((current, std::mem::take(&mut current_group)));
                    current_repo = Some(repo_id);
                }
            } else {
                current_repo = Some(repo_id);
            }

            current_group.push((reference.to_owned(), items));
        }

        if let Some(repo) = current_repo {
            if !current_group.is_empty() {
                result.push((repo, current_group));
            }
        }

        Ok(result)
    }
}
