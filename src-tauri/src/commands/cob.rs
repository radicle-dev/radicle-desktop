pub mod issue;
pub mod patch;

mod query {
    use serde::{Deserialize, Serialize};

    use radicle::issue;
    use radicle::patch;

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum IssueStatus {
        Closed,
        #[default]
        Open,
        All,
    }

    impl IssueStatus {
        pub fn matches(&self, issue: &issue::State) -> bool {
            match self {
                Self::Open => matches!(issue, issue::State::Open),
                Self::Closed => matches!(issue, issue::State::Closed { .. }),
                Self::All => true,
            }
        }
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum PatchStatus {
        #[default]
        Open,
        Draft,
        Archived,
        Merged,
        All,
    }

    impl PatchStatus {
        pub fn matches(&self, patch: &patch::State) -> bool {
            match self {
                Self::Open => matches!(patch, patch::State::Open { .. }),
                Self::Draft => matches!(patch, patch::State::Draft),
                Self::Archived => matches!(patch, patch::State::Archived),
                Self::Merged => matches!(patch, patch::State::Merged { .. }),
                Self::All => true,
            }
        }
    }
}
