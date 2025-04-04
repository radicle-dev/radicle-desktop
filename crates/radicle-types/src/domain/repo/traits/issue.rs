use radicle::identity;
use radicle::issue::IssueId;

use crate::domain::repo::models::cobs;
use crate::error::Error;

pub trait RepoIssues {
    fn list_issues(
        &self,
        rid: identity::RepoId,
        status: Option<cobs::query::IssueStatus>,
    ) -> Result<Vec<cobs::issue::Issue>, Error>;

    fn issue_by_id(
        &self,
        rid: identity::RepoId,
        id: IssueId,
    ) -> Result<Option<cobs::issue::Issue>, Error>;

    fn comment_threads_by_issue_id(
        &self,
        rid: identity::RepoId,
        id: IssueId,
    ) -> Result<Option<Vec<cobs::thread::Thread>>, Error>;

    fn create_issue(
        &self,
        rid: identity::RepoId,
        new: cobs::issue::NewIssue,
        opts: cobs::CobOptions,
    ) -> Result<cobs::issue::Issue, Error>;

    fn edit_issue(
        &self,
        rid: identity::RepoId,
        cob_id: IssueId,
        action: cobs::issue::Action,
        opts: cobs::CobOptions,
    ) -> Result<cobs::issue::Issue, Error>;
}
