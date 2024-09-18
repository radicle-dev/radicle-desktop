use std::str::FromStr;

use radicle::cob::{EntryId, ObjectId};
use radicle::git::Oid;
use radicle::identity::RepoId;
use radicle::node::Handle;
use radicle::storage::ReadStorage;
use radicle::Node;

use crate::error::Error;
use crate::types::cobs::{CobOptions, NewIssueComment, NewPatchComment};
use crate::AppState;

#[tauri::command]
pub fn create_issue_comment(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    new: NewIssueComment,
    opts: CobOptions,
) -> Result<Oid, Error> {
    let mut node = Node::new(ctx.profile.socket());
    let signer = ctx.profile.signer()?;
    let issue_id = ObjectId::from_str(&new.id)?;
    let repo = ctx.profile.storage.repository(rid)?;
    let mut issues = ctx.profile.issues_mut(&repo)?;
    let mut issue = issues.get_mut(&issue_id)?;
    let id = new.reply_to.unwrap_or_else(|| {
        let (root_id, _) = issue.root();
        *root_id
    });
    let oid = issue.comment(new.body, id, new.embeds, &signer)?;

    if opts.announce() {
        node.announce_refs(rid)?;
    }

    Ok::<_, Error>(oid)
}

#[tauri::command]
pub fn create_patch_comment(
    ctx: tauri::State<AppState>,
    rid: RepoId,
    new: NewPatchComment,
    opts: CobOptions,
) -> Result<Oid, Error> {
    let mut node = Node::new(ctx.profile.socket());
    let signer = ctx.profile.signer()?;
    let patch_id = ObjectId::from_str(&new.id)?;
    let revision_id = EntryId::from_str(&new.revision)?;
    let repo = ctx.profile.storage.repository(rid)?;
    let mut patches = ctx.profile.patches_mut(&repo)?;
    let mut patch = patches.get_mut(&patch_id)?;
    let oid = patch.comment(
        revision_id.into(),
        new.body,
        new.reply_to,
        new.location.map(|l| l.into()),
        new.embeds,
        &signer,
    )?;

    if opts.announce() {
        node.announce_refs(rid)?;
    }

    Ok::<_, Error>(oid)
}
