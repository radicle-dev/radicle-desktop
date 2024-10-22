use base64::{engine::general_purpose::STANDARD, Engine as _};

use radicle::cob;
use radicle::git;
use radicle::identity;
use radicle::storage::{ReadRepository, ReadStorage};
use radicle_types as types;
use radicle_types::cobs::IssueAction;

use crate::{error, AppState};

pub mod draft;
pub mod issue;
pub mod patch;

#[derive(serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IssueOp {
    #[ts(as = "String")]
    pub entry_id: git::Oid,
    #[serde(flatten)]
    pub action: types::cobs::IssueAction,
    #[ts(type = "number")]
    pub timestamp: cob::Timestamp,
    pub author: types::cobs::Author,
}

#[tauri::command]
pub async fn get_file_by_oid(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    oid: git::Oid,
) -> Result<String, error::Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let blob = repo.blob(oid)?;

    Ok::<_, error::Error>(STANDARD.encode(blob.content()))
}

#[tauri::command]
pub async fn save_embed(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    name: &str,
    bytes: &[u8],
) -> Result<git::Oid, error::Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let embed = cob::Embed::<git::Oid>::store(name, bytes, &repo.backend)?;

    Ok::<_, error::Error>(embed.oid())
}

#[tauri::command]
pub fn activity_by_id(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    type_name: cob::TypeName,
    id: git::Oid,
) -> Result<Vec<IssueOp>, error::Error> {
    let aliases = ctx.profile.aliases();
    let repo = ctx.profile.storage.repository(rid)?;
    let ops = cob::store::ops(&id.into(), &type_name, &repo).unwrap();
    let mut actions: Vec<IssueOp> = Vec::new();

    for op in ops.into_iter() {
        actions.extend(
            op.actions
                .iter()
                .filter_map(|action: &Vec<u8>| -> Option<IssueOp> {
                    let action: IssueAction = serde_json::from_slice(action).ok()?;

                    Some(IssueOp {
                        entry_id: op.id,
                        action,
                        author: types::cobs::Author::new(op.author.into(), &aliases),
                        timestamp: op.timestamp,
                    })
                }),
        )
    }

    Ok::<_, error::Error>(actions)
}

mod query {
    use serde::{Deserialize, Serialize};

    use radicle::issue;
    use radicle::patch;

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum IssueStatus {
        Closed,
        #[default]
        Open,
        All,
    }

    impl IssueStatus {
        pub fn matches(&self, issue: &issue::State) -> bool {
            match self {
                Self::Open => matches!(issue, issue::State::Open),
                Self::Closed => matches!(issue, issue::State::Closed { .. }),
                Self::All => true,
            }
        }
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum PatchStatus {
        #[default]
        Open,
        Draft,
        Archived,
        Merged,
    }

    impl From<patch::Status> for PatchStatus {
        fn from(value: patch::Status) -> Self {
            match value {
                patch::Status::Archived => Self::Archived,
                patch::Status::Draft => Self::Draft,
                patch::Status::Merged => Self::Merged,
                patch::Status::Open => Self::Open,
            }
        }
    }
    impl From<PatchStatus> for patch::Status {
        fn from(value: PatchStatus) -> Self {
            match value {
                PatchStatus::Archived => Self::Archived,
                PatchStatus::Draft => Self::Draft,
                PatchStatus::Merged => Self::Merged,
                PatchStatus::Open => Self::Open,
            }
        }
    }
}
