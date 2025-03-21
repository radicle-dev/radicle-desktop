use radicle::git;
use radicle::identity;
use radicle::issue::IssueId;
use radicle::patch;
use radicle::patch::Patch;
use radicle::patch::ReviewId;
use radicle::patch::{PatchId, RevisionId};
use serde::de::DeserializeOwned;

use crate::error::Error;

use super::traits::cobs::RepoActivity;
use super::traits::issue::RepoIssues;
use super::traits::patch::RepoPatches;
use super::traits::patch::RepoPatchesLister;
use super::traits::repo::RepoStorage;
use super::traits::thread::RepoThreads;
use super::traits::RepoService;

#[derive(Debug, Clone)]
pub struct Service<R, S>
where
    R: RepoActivity,
    R: RepoPatches,
    R: RepoIssues,
    R: RepoThreads,
    R: RepoStorage,
    S: RepoPatchesLister,
{
    cobs: R,
    patches: R,
    issues: R,
    threads: R,
    storage: R,
    patches_lister: S,
}

impl<R, S> Service<R, S>
where
    R: RepoActivity + Clone,
    R: RepoPatches + Clone,
    R: RepoIssues + Clone,
    R: RepoThreads + Clone,
    R: RepoStorage + Clone,
    S: RepoPatchesLister,
{
    pub fn new(radicle: R, sqlite: S) -> Self {
        Self {
            cobs: radicle.clone(),
            patches: radicle.clone(),
            issues: radicle.clone(),
            storage: radicle.clone(),
            threads: radicle,
            patches_lister: sqlite,
        }
    }
}

impl<R, S> RepoService for Service<R, S>
where
    R: RepoActivity + Clone,
    R: RepoPatches + Clone,
    R: RepoIssues + Clone,
    R: RepoThreads + Clone,
    R: RepoStorage + Clone,
    S: RepoPatchesLister,
{
    fn create_repo(&self, name: String, description: String) -> Result<(), Error> {
        self.storage.create_repo(name, description)
    }
    fn get_embed(
        &self,
        rid: identity::RepoId,
        name: Option<String>,
        oid: git::Oid,
    ) -> Result<super::models::cobs::EmbedWithMimeType, Error> {
        self.threads.get_embed(rid, name, oid)
    }

    fn create_issue(
        &self,
        rid: identity::RepoId,
        new: super::models::cobs::issue::NewIssue,
        opts: super::models::cobs::CobOptions,
    ) -> Result<super::models::cobs::issue::Issue, Error> {
        self.issues.create_issue(rid, new, opts)
    }

    fn edit_issue(
        &self,
        rid: identity::RepoId,
        cob_id: IssueId,
        action: super::models::cobs::issue::Action,
        opts: super::models::cobs::CobOptions,
    ) -> Result<super::models::cobs::issue::Issue, Error> {
        self.issues.edit_issue(rid, cob_id, action, opts)
    }

    fn issue_by_id(
        &self,
        rid: identity::RepoId,
        id: IssueId,
    ) -> Result<Option<super::models::cobs::issue::Issue>, Error> {
        self.issues.issue_by_id(rid, id)
    }

    fn list_issues(
        &self,
        rid: identity::RepoId,
        status: Option<super::models::cobs::query::IssueStatus>,
    ) -> Result<Vec<super::models::cobs::issue::Issue>, Error> {
        self.issues.list_issues(rid, status)
    }

    fn edit_patch(
        &self,
        rid: identity::RepoId,
        cob_id: PatchId,
        action: super::models::cobs::patch::Action,
        opts: super::models::cobs::CobOptions,
    ) -> Result<super::models::cobs::patch::Patch, Error> {
        self.patches.edit_patch(rid, cob_id, action, opts)
    }

    fn review_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: RevisionId,
        review_id: ReviewId,
    ) -> Result<Option<super::models::cobs::patch::Review>, Error> {
        self.patches.review_by_id(rid, id, revision_id, review_id)
    }

    fn get_patch_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
    ) -> Result<Option<super::models::cobs::patch::Patch>, Error> {
        self.patches.get_patch_by_id(rid, id)
    }

    fn revisions_by_patch(
        &self,
        rid: identity::RepoId,
        id: PatchId,
    ) -> Result<Option<Vec<super::models::cobs::patch::Revision>>, Error> {
        self.patches.revisions_by_patch(rid, id)
    }

    fn revision_by_patch_and_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: RevisionId,
    ) -> Result<Option<super::models::cobs::patch::Revision>, Error> {
        self.patches.revision_by_patch_and_id(rid, id, revision_id)
    }

    fn revision_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: RevisionId,
    ) -> Result<Option<super::models::cobs::patch::Revision>, Error> {
        self.patches.revision_by_id(rid, id, revision_id)
    }

    fn list_patches(
        &self,
        rid: identity::RepoId,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, super::models::cobs::patch::ListPatchesError>
    {
        self.patches_lister.list(rid)
    }

    fn list_patches_by_status(
        &self,
        rid: identity::RepoId,
        status: patch::Status,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, super::models::cobs::patch::ListPatchesError>
    {
        self.patches_lister.list_by_status(rid, status)
    }

    fn list_repos(
        &self,
        show: super::models::repo::Show,
    ) -> Result<Vec<super::models::repo::RepoInfo>, Error> {
        self.storage.list_repos(show)
    }

    fn repo_count(&self) -> Result<super::models::repo::RepoCount, Error> {
        self.storage.repo_count()
    }

    fn repo_by_id(&self, rid: identity::RepoId) -> Result<super::models::repo::RepoInfo, Error> {
        self.storage.repo_by_id(rid)
    }

    fn diff_stats(
        &self,
        rid: identity::RepoId,
        base: git::Oid,
        head: git::Oid,
    ) -> Result<super::models::cobs::Stats, Error> {
        self.storage.diff_stats(rid, base, head)
    }

    fn get_diff(
        &self,
        rid: identity::RepoId,
        options: super::models::diff::Options,
    ) -> Result<super::models::diff::Diff, Error> {
        self.storage.get_diff(rid, options)
    }

    fn list_commits(
        &self,
        rid: identity::RepoId,
        base: git::Oid,
        head: git::Oid,
    ) -> Result<Vec<super::models::repo::Commit>, Error> {
        self.storage.list_commits(rid, base, head)
    }

    fn save_embed_to_disk(
        &self,
        rid: identity::RepoId,
        oid: git::Oid,
        path: std::path::PathBuf,
    ) -> Result<(), Error> {
        self.threads.save_embed_to_disk(rid, oid, path)
    }

    fn save_embed_by_path(
        &self,
        rid: identity::RepoId,
        path: std::path::PathBuf,
    ) -> Result<git::Oid, Error> {
        self.threads.save_embed_by_path(rid, path)
    }

    fn save_embed_by_bytes(
        &self,
        rid: identity::RepoId,
        name: String,
        bytes: Vec<u8>,
    ) -> Result<git::Oid, Error> {
        self.threads.save_embed_by_bytes(rid, name, bytes)
    }

    fn create_issue_comment(
        &self,
        rid: identity::RepoId,
        new: super::models::cobs::thread::NewIssueComment,
        opts: super::models::cobs::CobOptions,
    ) -> Result<super::models::cobs::thread::Comment<super::models::cobs::Never>, Error> {
        self.threads.create_issue_comment(rid, new, opts)
    }

    fn create_patch_comment(
        &self,
        rid: identity::RepoId,
        new: super::models::cobs::thread::NewPatchComment,
        opts: super::models::cobs::CobOptions,
    ) -> Result<
        super::models::cobs::thread::Comment<super::models::cobs::thread::CodeLocation>,
        Error,
    > {
        self.threads.create_patch_comment(rid, new, opts)
    }

    fn comment_threads_by_issue_id(
        &self,
        rid: identity::RepoId,
        id: IssueId,
    ) -> Result<Option<Vec<super::models::cobs::thread::Thread>>, Error> {
        self.storage.comment_threads_by_issue_id(rid, id)
    }

    fn activity_by_id<A: DeserializeOwned, B: super::models::cobs::FromRadicleAction<A>>(
        &self,
        rid: identity::RepoId,
        type_name: &radicle::cob::TypeName,
        id: git::Oid,
    ) -> Result<Vec<super::models::cobs::Operation<B>>, Error> {
        self.cobs.activity_by_id(rid, type_name, id)
    }
}
