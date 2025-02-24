use std::collections::BTreeSet;

use radicle::node::Handle;
use radicle::patch::cache::Patches as _;
use radicle::storage::ReadStorage;
use radicle::{cob, git, identity, Node};

use crate::cobs;
use crate::domain::patch::models;
use crate::error::Error;
use crate::traits::Profile;

pub trait Patches: Profile {
    fn get_patch(
        &self,
        rid: identity::RepoId,
        id: git::Oid,
    ) -> Result<Option<models::patch::Patch>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo)?;
        let patch = patches.get(&id.into())?;
        let aliases = &profile.aliases();
        let patches = patch.map(|patch| models::patch::Patch::new(id.into(), &patch, aliases));

        Ok::<_, Error>(patches)
    }

    fn revisions_by_patch(
        &self,
        rid: identity::RepoId,
        id: git::Oid,
    ) -> Result<Option<Vec<models::patch::Revision>>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo)?;
        let revisions = patches.get(&id.into())?.map(|patch| {
            let aliases = &profile.aliases();

            patch
                .revisions()
                .map(|(_, r)| models::patch::Revision::new(r.clone(), aliases))
                .collect::<Vec<_>>()
        });

        Ok::<_, Error>(revisions)
    }

    fn revision_by_id(
        &self,
        rid: identity::RepoId,
        id: git::Oid,
        revision_id: git::Oid,
    ) -> Result<Option<models::patch::Revision>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo)?;
        let revision = patches.get(&id.into())?.and_then(|patch| {
            let aliases = &profile.aliases();

            patch
                .revision(&revision_id.into())
                .map(|r| models::patch::Revision::new(r.clone(), aliases))
        });

        Ok::<_, Error>(revision)
    }

    fn review_by_id(
        &self,
        rid: identity::RepoId,
        id: git::Oid,
        revision_id: git::Oid,
        review_id: cob::patch::ReviewId,
    ) -> Result<Option<models::patch::Review>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo)?;
        let review = patches.get(&id.into())?.and_then(|patch| {
            let aliases = &profile.aliases();

            patch
                .reviews_of(revision_id.into())
                .find(|(id, _)| *id == &review_id)
                .map(|(_, review)| models::patch::Review::new(review.clone(), aliases))
        });

        Ok::<_, Error>(review)
    }
}

pub trait PatchesMut: Profile {
    fn edit_patch(
        &self,
        rid: identity::RepoId,
        cob_id: git::Oid,
        action: models::patch::Action,
        opts: cobs::CobOptions,
    ) -> Result<models::patch::Patch, Error> {
        let profile = self.profile();
        let mut node = Node::new(profile.socket());
        let repo = profile.storage.repository(rid)?;
        let signer = profile.signer()?;
        let aliases = profile.aliases();
        let mut patches = profile.patches_mut(&repo)?;
        let mut patch = patches.get_mut(&cob_id.into())?;

        match action {
            models::patch::Action::RevisionEdit {
                revision,
                description,
                embeds,
            } => {
                patch.edit_revision(
                    revision,
                    description,
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            models::patch::Action::RevisionCommentRedact { revision, comment } => {
                patch.comment_redact(revision, comment, &signer)?;
            }
            models::patch::Action::ReviewCommentRedact { review, comment } => {
                patch.redact_review_comment(review, comment, &signer)?;
            }
            models::patch::Action::ReviewCommentReact {
                review,
                comment,
                reaction,
                active,
            } => {
                patch.react_review_comment(review, comment, reaction, active, &signer)?;
            }
            models::patch::Action::ReviewCommentResolve { review, comment } => {
                patch.resolve_review_comment(review, comment, &signer)?;
            }
            models::patch::Action::ReviewCommentUnresolve { review, comment } => {
                patch.unresolve_review_comment(review, comment, &signer)?;
            }
            models::patch::Action::Edit { title, target } => {
                patch.edit(title, target, &signer)?;
            }
            models::patch::Action::ReviewEdit {
                review,
                summary,
                verdict,
                labels,
            } => {
                patch.review_edit(review, verdict.map(|v| v.into()), summary, labels, &signer)?;
            }
            models::patch::Action::Review {
                revision,
                summary,
                verdict,
                labels,
            } => {
                patch.review(
                    revision,
                    verdict.map(|v| v.into()),
                    summary,
                    labels,
                    &signer,
                )?;
            }
            models::patch::Action::ReviewRedact { review } => {
                patch.redact_review(review, &signer)?;
            }
            models::patch::Action::ReviewComment {
                review,
                body,
                location,
                reply_to,
                embeds,
            } => {
                patch.review_comment(
                    review,
                    body,
                    location.map(|l| l.into()),
                    reply_to,
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            models::patch::Action::ReviewCommentEdit {
                review,
                comment,
                body,
                embeds,
            } => {
                patch.edit_review_comment(
                    review,
                    comment,
                    body,
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            models::patch::Action::Lifecycle { state } => {
                patch.lifecycle(state, &signer)?;
            }
            models::patch::Action::Assign { assignees } => {
                patch.assign(
                    assignees.iter().map(|a| *a.did()).collect::<BTreeSet<_>>(),
                    &signer,
                )?;
            }
            models::patch::Action::Label { labels } => {
                patch.label(labels, &signer)?;
            }
            models::patch::Action::RevisionReact {
                revision,
                reaction,
                location,
                active,
            } => {
                patch.react(
                    revision,
                    reaction,
                    location.map(|l| l.into()),
                    active,
                    &signer,
                )?;
            }
            models::patch::Action::RevisionComment {
                revision,
                location,
                body,
                reply_to,
                embeds,
            } => {
                patch.comment(
                    revision,
                    body,
                    reply_to,
                    location.map(|l| l.into()),
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            models::patch::Action::RevisionCommentEdit {
                revision,
                comment,
                body,
                embeds,
            } => {
                patch.comment_edit(
                    revision,
                    comment,
                    body,
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            models::patch::Action::RevisionCommentReact {
                revision,
                comment,
                reaction,
                active,
            } => {
                patch.comment_react(revision, comment, reaction, active, &signer)?;
            }
            models::patch::Action::RevisionRedact { revision } => {
                patch.redact(revision, &signer)?;
            }
            models::patch::Action::Merge { .. } => {
                unimplemented!("We don't support merging of patches through the desktop")
            }
            models::patch::Action::Revision { .. } => {
                unimplemented!("We don't support creating new revisions through the desktop")
            }
        }

        if opts.announce() {
            if let Err(e) = node.announce_refs(rid) {
                log::error!("Not able to announce changes: {}", e)
            }
        }

        Ok::<_, Error>(models::patch::Patch::new(*patch.id(), &patch, &aliases))
    }
}
