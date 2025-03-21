use radicle::git;
use radicle::identity;

use crate::domain::repo::models::cobs;
use crate::error::Error;

pub trait RepoThreads {
    fn get_embed(
        &self,
        rid: identity::RepoId,
        name: Option<String>,
        oid: git::Oid,
    ) -> Result<cobs::EmbedWithMimeType, Error>;

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
        new: cobs::thread::NewIssueComment,
        opts: cobs::CobOptions,
    ) -> Result<cobs::thread::Comment<cobs::Never>, Error>;

    fn create_patch_comment(
        &self,
        rid: identity::RepoId,
        new: cobs::thread::NewPatchComment,
        opts: cobs::CobOptions,
    ) -> Result<cobs::thread::Comment<cobs::thread::CodeLocation>, Error>;
}
