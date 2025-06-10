use std::ops::ControlFlow;

use radicle::git;
use radicle::identity;
use radicle::storage::ReadStorage;

use radicle::issue::TYPENAME;
use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::cobs::Cobs;
use radicle_types::traits::issue::Issues;
use radicle_types::traits::issue::IssuesMut;

use crate::AppState;

#[tauri::command]
pub fn create_issue(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    new: types::cobs::issue::NewIssue,
    opts: types::cobs::CobOptions,
) -> Result<types::cobs::issue::Issue, Error> {
    ctx.create_issue(rid, new, opts)
}

#[tauri::command]
pub fn edit_issue(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    cob_id: git::Oid,
    action: types::cobs::issue::Action,
    opts: types::cobs::CobOptions,
) -> Result<types::cobs::issue::Issue, Error> {
    ctx.edit_issue(rid, cob_id, action, opts)
}

#[tauri::command]
pub(crate) fn list_issues(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    status: Option<types::cobs::query::IssueStatus>,
) -> Result<Vec<types::cobs::issue::Issue>, Error> {
    ctx.list_issues(rid, status)
}

#[tauri::command]
pub(crate) fn issue_by_id(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Option<types::cobs::issue::Issue>, Error> {
    ctx.issue_by_id(rid, id)
}

#[tauri::command]
pub(crate) fn comment_threads_by_issue_id(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Option<Vec<types::cobs::thread::Thread>>, Error> {
    ctx.comment_threads_by_issue_id(rid, id)
}

#[tauri::command]
pub fn activity_by_issue(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Vec<types::cobs::Operation<types::cobs::issue::Action>>, Error> {
    ctx.activity_by_id(rid, &TYPENAME, id)
}

#[tauri::command]
pub async fn rebuild_issue_cache(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    on_event: tauri::ipc::Channel<types::cobs::CacheEvent>,
) -> Result<(), Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let mut issues = ctx.profile.issues_mut(&repo)?;
    on_event.send(types::cobs::CacheEvent::Started { rid })?;
    issues.write_all(|result, progress| {
        match result {
            Ok((id, _)) => {
                if on_event
                    .send(types::cobs::CacheEvent::Progress {
                        rid,
                        oid: **id,
                        current: progress.current(),
                        total: progress.total(),
                    })
                    .is_err()
                {
                    log::error!("Failed to send progress");
                }
            }
            Err(err) => log::warn!("Failed to retrieve issue: {err}"),
        };
        ControlFlow::Continue(())
    })?;
    on_event.send(types::cobs::CacheEvent::Finished { rid })?;

    Ok(())
}
