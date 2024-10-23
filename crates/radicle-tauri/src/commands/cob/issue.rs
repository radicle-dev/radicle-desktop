use radicle::git;
use radicle::identity;
use radicle::issue::cache::Issues;
use radicle::node::Handle;
use radicle::node::Node;
use radicle::storage::ReadStorage;
use radicle_types as types;

use crate::cob::query;
use crate::error::Error;
use crate::AppState;

#[tauri::command]
pub fn create_issue(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    new: types::cobs::issue::NewIssue,
    opts: types::cobs::CobOptions,
) -> Result<types::cobs::issue::Issue, Error> {
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
        new.embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
        &signer,
    )?;

    if opts.announce() {
        node.announce_refs(rid)?;
    }

    Ok::<_, Error>(types::cobs::issue::Issue::new(issue.id(), &issue, &aliases))
}

#[tauri::command]
pub fn edit_issue(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    cob_id: git::Oid,
    action: types::cobs::issue::Action,
    opts: types::cobs::CobOptions,
) -> Result<types::cobs::issue::Issue, Error> {
    let mut node = Node::new(ctx.profile.socket());
    let repo = ctx.profile.storage.repository(rid)?;
    let signer = ctx.profile.signer()?;
    let aliases = ctx.profile.aliases();
    let mut issues = ctx.profile.issues_mut(&repo)?;
    let mut issue = issues.get_mut(&cob_id.into())?;

    match action {
        types::cobs::issue::Action::Lifecycle { state } => {
            issue.lifecycle(state.into(), &signer)?;
        }
        types::cobs::issue::Action::Assign { assignees } => {
            issue.assign(assignees, &signer)?;
        }
        types::cobs::issue::Action::Label { labels } => {
            issue.label(labels, &signer)?;
        }
        types::cobs::issue::Action::CommentReact {
            id,
            reaction,
            active,
        } => {
            issue.react(id, reaction, active, &signer)?;
        }
        types::cobs::issue::Action::CommentRedact { id } => {
            issue.redact_comment(id, &signer)?;
        }
        types::cobs::issue::Action::Comment {
            body,
            reply_to,
            embeds,
        } => {
            issue.comment(
                body,
                reply_to.unwrap_or(cob_id),
                embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
                &signer,
            )?;
        }
        types::cobs::issue::Action::CommentEdit { id, body, embeds } => {
            issue.edit_comment(
                id,
                body,
                embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
                &signer,
            )?;
        }
        types::cobs::issue::Action::Edit { title } => {
            issue.edit(title, &signer)?;
        }
    }

    if opts.announce() {
        node.announce_refs(rid)?;
    }

    Ok::<_, Error>(types::cobs::issue::Issue::new(issue.id(), &issue, &aliases))
}

#[tauri::command]
pub fn list_issues(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    status: query::IssueStatus,
) -> Result<Vec<types::cobs::issue::Issue>, Error> {
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
        .map(|(id, issue)| types::cobs::issue::Issue::new(&id, &issue, aliases))
        .collect::<Vec<_>>();

    Ok::<_, Error>(issues)
}

#[tauri::command]
pub fn issue_by_id(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    id: git::Oid,
) -> Result<Option<types::cobs::issue::Issue>, Error> {
    let repo = ctx.profile.storage.repository(rid)?;
    let issues = ctx.profile.issues(&repo)?;
    let issue = issues.get(&id.into())?;

    let aliases = &ctx.profile.aliases();
    let issue = issue.map(|issue| types::cobs::issue::Issue::new(&id.into(), &issue, aliases));

    Ok::<_, Error>(issue)
}
