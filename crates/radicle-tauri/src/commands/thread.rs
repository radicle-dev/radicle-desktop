use radicle::identity;

use radicle_types::domain::repo::models::cobs;
use radicle_types::domain::repo::service::Service;
use radicle_types::domain::repo::traits::RepoService;
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle as R;
use radicle_types::outbound::sqlite::Sqlite as S;

#[tauri::command]
pub fn create_issue_comment(
    service: tauri::State<'_, Service<R, S>>,
    rid: identity::RepoId,
    new: cobs::thread::NewIssueComment,
    opts: cobs::CobOptions,
) -> Result<cobs::thread::Comment<cobs::Never>, Error> {
    service.create_issue_comment(rid, new, opts)
}

#[tauri::command]
pub fn create_patch_comment(
    service: tauri::State<'_, Service<R, S>>,
    rid: identity::RepoId,
    new: cobs::thread::NewPatchComment,
    opts: cobs::CobOptions,
) -> Result<cobs::thread::Comment<cobs::thread::CodeLocation>, Error> {
    service.create_patch_comment(rid, new, opts)
}
