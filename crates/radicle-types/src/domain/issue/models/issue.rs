use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListIssuesError {
    #[error(transparent)]
    Sqlite(#[from] sqlite::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    // to be extended as new error scenarios are introduced
}
