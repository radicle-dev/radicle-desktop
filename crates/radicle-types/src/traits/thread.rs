use base64::{engine::general_purpose::STANDARD, Engine as _};
use localtime::LocalTime;

use radicle::cob;
use radicle::git;
use radicle::identity;
use radicle::node::Handle;
use radicle::storage::ReadRepository;
use radicle::storage::ReadStorage;
use radicle::Node;

use crate::cobs;
use crate::error::Error;
use crate::traits::Profile;

pub trait Thread: Profile {
    fn get_embed(&self, rid: identity::RepoId, oid: git::Oid) -> Result<String, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let blob = repo.blob(oid)?;

        Ok::<_, Error>(STANDARD.encode(blob.content()))
    }

    fn save_embed(
        &self,
        rid: identity::RepoId,
        name: &str,
        bytes: &[u8],
    ) -> Result<git::Oid, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let embed = radicle::cob::Embed::<git::Oid>::store(name, bytes, &repo.backend)?;

        Ok(embed.oid())
    }

    fn create_issue_comment(
        &self,
        rid: identity::RepoId,
        new: cobs::thread::NewIssueComment,
        opts: cobs::CobOptions,
    ) -> Result<cobs::thread::Comment<cobs::Never>, Error> {
        let profile = self.profile();
        let aliases = &profile.aliases();
        let mut node = Node::new(profile.socket());
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;
        let mut issues = profile.issues_mut(&repo)?;
        let mut issue = issues.get_mut(&new.id.into())?;
        let id = new.reply_to.unwrap_or_else(|| {
            let (root_id, _) = issue.root();
            *root_id
        });
        let n = new.clone();
        let oid = issue.comment(
            n.body,
            id,
            n.embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
            &signer,
        )?;

        if opts.announce() {
            if let Err(e) = node.announce_refs(rid) {
                eprintln!("Not able to announce changes: {}", e)
            }
        }

        Ok(cobs::thread::Comment::<cobs::Never>::new(
            oid,
            cob::thread::Comment::new(
                *signer.public_key(),
                new.body,
                id.into(),
                None,
                new.embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                LocalTime::now().into(),
            ),
            aliases,
        ))
    }

    fn create_patch_comment(
        &self,
        rid: identity::RepoId,
        new: cobs::thread::NewPatchComment,
        opts: cobs::CobOptions,
    ) -> Result<cobs::thread::Comment<cobs::thread::CodeLocation>, Error> {
        let profile = self.profile();
        let aliases = &profile.aliases();
        let mut node = Node::new(profile.socket());
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;
        let mut patches = profile.patches_mut(&repo)?;
        let mut patch = patches.get_mut(&new.id.into())?;
        let n = new.clone();
        let oid = patch.comment(
            new.revision.into(),
            n.body,
            n.reply_to,
            n.location.map(|l| l.into()),
            n.embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
            &signer,
        )?;

        if opts.announce() {
            if let Err(e) = node.announce_refs(rid) {
                eprintln!("Not able to announce changes: {}", e)
            }
        }

        Ok(cobs::thread::Comment::<cobs::thread::CodeLocation>::new(
            oid,
            cob::thread::Comment::new(
                *signer.public_key(),
                new.body,
                new.reply_to,
                new.location.map(|l| l.into()),
                new.embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                LocalTime::now().into(),
            ),
            aliases,
        ))
    }
}
