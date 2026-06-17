use radicle::identity;
use radicle::issue::{Issue, IssueId};

use crate::domain::issue::models;

pub trait IssueStorage {
    fn list(
        &self,
        rid: identity::RepoId,
    ) -> Result<impl Iterator<Item = (IssueId, Issue)>, models::issue::ListIssuesError>;

    fn list_by_status(
        &self,
        rid: identity::RepoId,
        status: &'static str,
    ) -> Result<impl Iterator<Item = (IssueId, Issue)>, models::issue::ListIssuesError>;
}

pub trait IssueService {
    fn list(
        &self,
        rid: identity::RepoId,
    ) -> Result<impl Iterator<Item = (IssueId, Issue)>, models::issue::ListIssuesError>;

    fn list_by_status(
        &self,
        rid: identity::RepoId,
        status: &'static str,
    ) -> Result<impl Iterator<Item = (IssueId, Issue)>, models::issue::ListIssuesError>;
}
