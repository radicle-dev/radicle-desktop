use radicle::issue::IssueId;
use radicle::patch::{Patch, PatchId, ReviewId, RevisionId};
use radicle::{git, identity};
use serde::de::DeserializeOwned;

use crate::domain::repo::models;
use crate::error::Error;

use super::models::cobs::CobOptions;

pub mod cobs;
pub mod issue;
pub mod patch;
pub mod repo;
pub mod thread;

pub trait RepoService {
    fn create_repo(
        &self,
        name: String,
        description: String,
        default_branch: git::RefString,
        signature: Option<radicle::git::raw::Signature>,
    ) -> Result<(), Error>;
    fn comment_threads_by_issue_id(
        &self,
        rid: identity::RepoId,
        id: IssueId,
    ) -> Result<Option<Vec<models::cobs::thread::Thread>>, Error>;

    fn activity_by_id<A: DeserializeOwned, B: models::cobs::FromRadicleAction<A>>(
        &self,
        rid: identity::RepoId,
        type_name: &radicle::cob::TypeName,
        id: git::Oid,
    ) -> Result<Vec<models::cobs::Operation<B>>, Error>;

    fn list_repos(&self, show: models::repo::Show) -> Result<Vec<models::repo::RepoInfo>, Error>;
    fn repo_count(&self) -> Result<models::repo::RepoCount, Error>;
    fn repo_by_id(&self, rid: identity::RepoId) -> Result<models::repo::RepoInfo, Error>;
    fn diff_stats(
        &self,
        rid: identity::RepoId,
        base: git::Oid,
        head: git::Oid,
    ) -> Result<models::diff::Stats, Error>;

    fn get_diff(
        &self,
        rid: identity::RepoId,
        options: models::diff::DiffOptions,
    ) -> Result<models::diff::Diff, Error>;

    fn list_commits(
        &self,
        rid: identity::RepoId,
        base: git::Oid,
        head: git::Oid,
    ) -> Result<Vec<models::repo::Commit>, Error>;

    fn get_embed(
        &self,
        rid: identity::RepoId,
        name: Option<String>,
        oid: git::Oid,
    ) -> Result<models::cobs::EmbedWithMimeType, Error>;

    fn save_embed_to_disk(
        &self,
        rid: identity::RepoId,
        oid: git::Oid,
        path: std::path::PathBuf,
    ) -> Result<(), Error>;

    fn save_embed_by_path(
        &self,
        rid: identity::RepoId,
        path: std::path::PathBuf,
    ) -> Result<git::Oid, Error>;

    fn save_embed_by_bytes(
        &self,
        rid: identity::RepoId,
        name: String,
        bytes: Vec<u8>,
    ) -> Result<git::Oid, Error>;

    fn create_issue_comment(
        &self,
        rid: identity::RepoId,
        new: models::cobs::thread::NewIssueComment,
        opts: models::cobs::CobOptions,
    ) -> Result<models::cobs::thread::Comment<models::cobs::Never>, Error>;

    fn create_patch_comment(
        &self,
        rid: identity::RepoId,
        new: models::cobs::thread::NewPatchComment,
        opts: models::cobs::CobOptions,
    ) -> Result<models::cobs::thread::Comment<models::cobs::thread::CodeLocation>, Error>;

    fn create_issue(
        &self,
        rid: identity::RepoId,
        new: models::cobs::issue::NewIssue,
        opts: models::cobs::CobOptions,
    ) -> Result<models::cobs::issue::Issue, Error>;

    fn edit_issue(
        &self,
        rid: identity::RepoId,
        cob_id: IssueId,
        action: models::cobs::issue::Action,
        opts: models::cobs::CobOptions,
    ) -> Result<models::cobs::issue::Issue, Error>;

    fn list_issues(
        &self,
        rid: identity::RepoId,
        status: Option<models::cobs::query::IssueStatus>,
    ) -> Result<Vec<models::cobs::issue::Issue>, Error>;

    fn issue_by_id(
        &self,
        rid: identity::RepoId,
        id: IssueId,
    ) -> Result<Option<models::cobs::issue::Issue>, Error>;

    fn list_patches(
        &self,
        rid: identity::RepoId,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, models::cobs::patch::ListPatchesError>;

    fn list_patches_by_status(
        &self,
        rid: identity::RepoId,
        status: radicle::patch::Status,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, models::cobs::patch::ListPatchesError>;

    fn get_patch_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
    ) -> Result<Option<models::cobs::patch::Patch>, Error>;

    fn revisions_by_patch(
        &self,
        rid: identity::RepoId,
        id: PatchId,
    ) -> Result<Option<Vec<models::cobs::patch::Revision>>, Error>;

    fn revision_by_patch_and_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: RevisionId,
    ) -> Result<Option<models::cobs::patch::Revision>, Error>;

    fn revision_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: RevisionId,
    ) -> Result<Option<models::cobs::patch::Revision>, Error>;

    fn review_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: RevisionId,
        review_id: ReviewId,
    ) -> Result<Option<models::cobs::patch::Review>, Error>;

    fn edit_patch(
        &self,
        rid: identity::RepoId,
        cob_id: PatchId,
        action: models::cobs::patch::Action,
        opts: CobOptions,
    ) -> Result<models::cobs::patch::Patch, Error>;
}
