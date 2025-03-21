use radicle::{git, identity};

use crate::domain::repo::models::{cobs, diff, repo};
use crate::error::Error;

pub trait RepoStorage {
    fn create_repo(&self, name: String, description: String) -> Result<(), Error>;
    fn list_repos(&self, show: repo::Show) -> Result<Vec<repo::RepoInfo>, Error>;
    fn repo_count(&self) -> Result<repo::RepoCount, Error>;
    fn repo_by_id(&self, rid: identity::RepoId) -> Result<repo::RepoInfo, Error>;
    fn diff_stats(
        &self,
        rid: identity::RepoId,
        base: git::Oid,
        head: git::Oid,
    ) -> Result<cobs::Stats, Error>;

    fn get_diff(&self, rid: identity::RepoId, options: diff::Options) -> Result<diff::Diff, Error>;

    fn list_commits(
        &self,
        rid: identity::RepoId,
        base: git::Oid,
        head: git::Oid,
    ) -> Result<Vec<repo::Commit>, Error>;
}
