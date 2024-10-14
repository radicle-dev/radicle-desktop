use radicle::git;
use radicle::identity::RepoId;
use radicle::issue::cache::Issues;
use radicle::node::Handle;
use radicle::node::Node;
use radicle::storage::ReadStorage;

use crate::cob::query;
use crate::error::Error;
use crate::types::cobs;
use crate::AppState;

#[tauri::command]
pub fn create_issue(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    new: cobs::NewIssue,
    opts: cobs::CobOptions,
) -> Result<cobs::Issue, Error> {
    let mut node = Node::new(ctx.profile.socket());
    let repo = ctx.profile.storage.repository(rid)?;
    let signer = ctx.profile.signer()?;
    let aliases = ctx.profile.aliases();
    let mut issues = ctx.profile.issues_mut(&repo)?;
    let issue = issues.create(
        new.title,
        new.description,
        &new.labels,
        &new.assignees,
        new.embeds,
        &signer,
    )?;

    if opts.announce() {
        node.announce_refs(rid)?;
    }

    Ok::<_, Error>(cobs::Issue::new(issue.id(), &issue, &aliases))
}

#[tauri::command]
pub fn edit_issue(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    cob_id: git::Oid,
    action: cobs::IssueAction,
    opts: cobs::CobOptions,
) -> Result<cobs::Issue, Error> {
    let mut node = Node::new(ctx.profile.socket());
    let repo = ctx.profile.storage.repository(rid)?;
    let signer = ctx.profile.signer()?;
    let aliases = ctx.profile.aliases();
    let mut issues = ctx.profile.issues_mut(&repo)?;
    let mut issue = issues.get_mut(&cob_id.into())?;

    match action {
        cobs::IssueAction::Lifecycle { state } => {
            issue.lifecycle(state, &signer)?;
        }
        cobs::IssueAction::Assign { assignees } => {
            issue.assign(assignees, &signer)?;
        }
        cobs::IssueAction::Label { labels } => {
            issue.label(labels, &signer)?;
        }
        cobs::IssueAction::CommentReact {
            id,
            reaction,
            active,
        } => {
            issue.react(id, reaction, active, &signer)?;
        }
        cobs::IssueAction::CommentRedact { id } => {
            issue.redact_comment(id, &signer)?;
        }
        cobs::IssueAction::Comment {
            body,
            reply_to,
            embeds,
        } => {
            issue.comment(body, reply_to.unwrap_or(cob_id), embeds, &signer)?;
        }
        cobs::IssueAction::CommentEdit { id, body, embeds } => {
            issue.edit_comment(id, body, embeds, &signer)?;
        }
        cobs::IssueAction::Edit { title } => {
            issue.edit(title, &signer)?;
        }
    }

    if opts.announce() {
        node.announce_refs(rid)?;
    }

    Ok::<_, Error>(cobs::Issue::new(issue.id(), &issue, &aliases))
}

#[tauri::command]
pub fn list_issues(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    status: query::IssueStatus,
) -> Result<Vec<cobs::Issue>, Error> {
    let repo = ctx.profile.storage.repository(rid)?;
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
        .map(|(id, issue)| cobs::Issue::new(&id, &issue, aliases))
        .collect::<Vec<_>>();

    Ok::<_, Error>(issues)
}

#[tauri::command]
pub fn issue_by_id(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    id: git::Oid,
) -> Result<Option<cobs::Issue>, Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let issues = ctx.profile.issues(&repo)?;
    let issue = issues.get(&id.into())?;

    let aliases = &ctx.profile.aliases();
    let issue = issue.map(|issue| cobs::Issue::new(&id.into(), &issue, aliases));

    Ok::<_, Error>(issue)
}
