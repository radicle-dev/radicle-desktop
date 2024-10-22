use localtime::LocalTime;

use radicle::cob;
use radicle::identity;
use radicle::node::Handle;
use radicle::storage::ReadStorage;
use radicle::Node;
use radicle_types::cobs;
use radicle_types::thread;

use crate::error::Error;
use crate::AppState;

#[tauri::command]
pub fn create_issue_comment(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    new: thread::NewIssueComment,
    opts: cobs::CobOptions,
) -> Result<thread::Comment<cobs::Never>, Error> {
    let aliases = &ctx.profile.aliases();
    let mut node = Node::new(ctx.profile.socket());
    let signer = ctx.profile.signer()?;
    let repo = ctx.profile.storage.repository(rid)?;
    let mut issues = ctx.profile.issues_mut(&repo)?;
    let mut issue = issues.get_mut(&new.id.into())?;
    let id = new.reply_to.unwrap_or_else(|| {
        let (root_id, _) = issue.root();
        *root_id
    });
    let oid = issue.comment(new.body.clone(), id, new.embeds.clone(), &signer)?;

    if opts.announce() {
        node.announce_refs(rid)?;
    }

    Ok(thread::Comment::<cobs::Never>::new(
        oid,
        cob::thread::Comment::new(
            *signer.public_key(),
            new.body,
            id.into(),
            None,
            new.embeds,
            LocalTime::now().into(),
        ),
        aliases,
    ))
}

#[tauri::command]
pub fn create_patch_comment(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    new: thread::NewPatchComment,
    opts: cobs::CobOptions,
) -> Result<thread::Comment<thread::CodeLocation>, Error> {
    let aliases = &ctx.profile.aliases();
    let mut node = Node::new(ctx.profile.socket());
    let signer = ctx.profile.signer()?;
    let repo = ctx.profile.storage.repository(rid)?;
    let mut patches = ctx.profile.patches_mut(&repo)?;
    let mut patch = patches.get_mut(&new.id.into())?;
    let n = new.clone();
    let oid = patch.comment(
        new.revision.into(),
        n.body,
        n.reply_to,
        n.location.map(|l| l.into()),
        n.embeds,
        &signer,
    )?;

    if opts.announce() {
        node.announce_refs(rid)?;
    }

    Ok(thread::Comment::<thread::CodeLocation>::new(
        oid,
        cob::thread::Comment::new(
            *signer.public_key(),
            new.body,
            new.reply_to,
            new.location.map(|l| l.into()),
            new.embeds,
            LocalTime::now().into(),
        ),
        aliases,
    ))
}
