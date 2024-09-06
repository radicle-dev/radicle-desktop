use radicle::identity::RepoId;
use radicle::issue::cache::Issues;
use radicle::patch::cache::Patches;

use crate::error::Error;
use crate::types::cobs;
use crate::AppState;

#[tauri::command]
pub fn list_issues(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    status: query::IssueStatus,
) -> Result<Vec<cobs::Issue>, Error> {
    let (repo, _) = ctx.repo(rid)?;
    let issues = ctx.profile.issues(&repo)?;
    let mut issues: Vec<_> = issues
        .list()?
        .filter_map(|r| {
            let (id, issue) = r.ok()?;
            (status.matches(issue.state())).then_some((id, issue))
        })
        .collect::<Vec<_>>();

    issues.sort_by(|(_, a), (_, b)| b.timestamp().cmp(&a.timestamp()));
    let aliases = &ctx.profile.aliases();
    let issues = issues
        .into_iter()
        .map(|(id, issue)| cobs::Issue::new(id, issue, aliases))
        .collect::<Vec<_>>();

    Ok::<_, Error>(issues)
}

#[tauri::command]
pub fn list_patches(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    status: query::PatchStatus,
) -> Result<Vec<cobs::Patch>, Error> {
    let (repo, _) = ctx.repo(rid)?;
    let patches = ctx.profile.patches(&repo)?;
    let mut patches: Vec<_> = patches
        .list()?
        .filter_map(|r| {
            let (id, patch) = r.ok()?;
            (status.matches(patch.state())).then_some((id, patch))
        })
        .collect::<Vec<_>>();

    patches.sort_by(|(_, a), (_, b)| b.timestamp().cmp(&a.timestamp()));
    let aliases = &ctx.profile.aliases();
    let patches = patches
        .into_iter()
        .map(|(id, patch)| cobs::Patch::new(id, patch, aliases))
        .collect::<Vec<_>>();

    Ok::<_, Error>(patches)
}

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
