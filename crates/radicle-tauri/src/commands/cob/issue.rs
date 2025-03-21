use radicle::git;
use radicle::identity;

use radicle::issue::IssueId;
use radicle::issue::TYPENAME;
use radicle_types::domain::repo::models::cobs;
use radicle_types::domain::repo::service::Service;
use radicle_types::domain::repo::traits::RepoService;
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle as R;
use radicle_types::outbound::sqlite::Sqlite as S;

#[tauri::command]
pub fn create_issue(
    service: tauri::State<Service<R, S>>,
    rid: identity::RepoId,
    new: cobs::issue::NewIssue,
    opts: cobs::CobOptions,
) -> Result<cobs::issue::Issue, Error> {
    service.create_issue(rid, new, opts)
}

#[tauri::command]
pub fn edit_issue(
    service: tauri::State<Service<R, S>>,
    rid: identity::RepoId,
    cob_id: git::Oid,
    action: cobs::issue::Action,
    opts: cobs::CobOptions,
) -> Result<cobs::issue::Issue, Error> {
    service.edit_issue(rid, cob_id.into(), action, opts)
}

#[tauri::command]
pub(crate) fn list_issues(
    service: tauri::State<Service<R, S>>,
    rid: identity::RepoId,
    status: Option<cobs::query::IssueStatus>,
) -> Result<Vec<cobs::issue::Issue>, Error> {
    service.list_issues(rid, status)
}

#[tauri::command]
pub(crate) fn issue_by_id(
    service: tauri::State<Service<R, S>>,
    rid: identity::RepoId,
    id: IssueId,
) -> Result<Option<cobs::issue::Issue>, Error> {
    service.issue_by_id(rid, id)
}

#[tauri::command]
pub(crate) fn comment_threads_by_issue_id(
    service: tauri::State<Service<R, S>>,
    rid: identity::RepoId,
    id: IssueId,
) -> Result<Option<Vec<cobs::thread::Thread>>, Error> {
    service.comment_threads_by_issue_id(rid, id)
}

#[tauri::command]
pub fn activity_by_issue(
    service: tauri::State<Service<R, S>>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Vec<cobs::Operation<cobs::issue::Action>>, Error> {
    service.activity_by_id(rid, &TYPENAME, id)
}
