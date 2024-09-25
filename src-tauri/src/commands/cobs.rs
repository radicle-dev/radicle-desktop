use std::str::FromStr;

use radicle::cob::ObjectId;
use radicle::git::Oid;
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
pub fn issues_by_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    id: Oid,
) -> Result<Option<cobs::Issue>, Error> {
    let (repo, _) = ctx.repo(rid)?;
    let issues = ctx.profile.issues(&repo)?;
    let issue = issues.get(&id.into())?;

    let aliases = &ctx.profile.aliases();
    let issue = issue.map(|issue| cobs::Issue::new(id.into(), issue, aliases));

    Ok::<_, Error>(issue)
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

#[tauri::command]
pub fn patches_by_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    id: String,
) -> Result<Option<cobs::Patch>, Error> {
    let id = ObjectId::from_str(&id)?;
    let (repo, _) = ctx.repo(rid)?;
    let patches = ctx.profile.patches(&repo)?;
    let patch = patches.get(&id)?;

    let aliases = &ctx.profile.aliases();
    let patches = patch.map(|patch| cobs::Patch::new(id, patch, aliases));

    Ok::<_, Error>(patches)
}

#[tauri::command]
pub fn revisions_by_patch(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    id: String,
) -> Result<Option<Vec<cobs::Revision>>, Error> {
    let id = ObjectId::from_str(&id)?;
    let (repo, _) = ctx.repo(rid)?;
    let patches = ctx.profile.patches(&repo)?;

    let revisions = patches.get(&id)?.map(|patch| {
        let aliases = &ctx.profile.aliases();

        patch
            .revisions()
            .map(|(_, r)| cobs::Revision::new(r.clone(), aliases))
            .collect::<Vec<_>>()
    });

    Ok::<_, Error>(revisions)
}

#[tauri::command]
pub fn revisions_by_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    id: String,
    revision_id: String,
) -> Result<Option<cobs::Revision>, Error> {
    let id = ObjectId::from_str(&id)?;
    let (repo, _) = ctx.repo(rid)?;
    let patches = ctx.profile.patches(&repo)?;
    let revision = patches.get(&id)?.and_then(|patch| {
        let revision_id = Oid::from_str(&revision_id).ok()?;
        let aliases = &ctx.profile.aliases();

        patch
            .revision(&revision_id.into())
            .map(|r| cobs::Revision::new(r.clone(), aliases))
    });
    Ok::<_, Error>(revision)
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
