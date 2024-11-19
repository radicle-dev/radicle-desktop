use radicle::git;
use radicle::identity;

use radicle_types as types;
use radicle_types::error::Error;
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
