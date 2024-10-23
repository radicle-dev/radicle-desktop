use localtime::LocalTime;

use radicle::cob;
use radicle::identity;
use radicle::node::Handle;
use radicle::storage::ReadStorage;
use radicle::Node;
use radicle_types as types;

use crate::error::Error;
use crate::AppState;

#[tauri::command]
pub fn create_issue_comment(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    new: types::cobs::thread::NewIssueComment,
    opts: types::cobs::CobOptions,
) -> Result<types::cobs::thread::Comment<types::cobs::Never>, Error> {
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
    let n = new.clone();
    let oid = issue.comment(
        n.body,
        id,
        n.embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
        &signer,
    )?;

    if opts.announce() {
        node.announce_refs(rid)?;
    }

    Ok(types::cobs::thread::Comment::<types::cobs::Never>::new(
        oid,
        cob::thread::Comment::new(
            *signer.public_key(),
            new.body,
            id.into(),
            None,
            new.embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
            LocalTime::now().into(),
        ),
        aliases,
    ))
}

#[tauri::command]
pub fn create_patch_comment(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    new: types::cobs::thread::NewPatchComment,
    opts: types::cobs::CobOptions,
) -> Result<types::cobs::thread::Comment<types::cobs::thread::CodeLocation>, Error> {
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
        n.embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
        &signer,
    )?;

    if opts.announce() {
        node.announce_refs(rid)?;
    }

    Ok(types::cobs::thread::Comment::<
        types::cobs::thread::CodeLocation,
    >::new(
        oid,
        cob::thread::Comment::new(
            *signer.public_key(),
            new.body,
            new.reply_to,
            new.location.map(|l| l.into()),
            new.embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
            LocalTime::now().into(),
        ),
        aliases,
    ))
}
