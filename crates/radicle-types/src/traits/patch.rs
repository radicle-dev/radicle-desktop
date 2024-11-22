use radicle::cob::migrate;
use radicle::node::Handle;
use radicle::patch::cache::Patches as _;
use radicle::storage;
use radicle::storage::ReadStorage;
use radicle::{cob, git, identity, patch, Node};

use crate::cobs;
use crate::error::Error;
use crate::traits::Profile;

pub trait Patches: Profile {
    fn list_patches(
        &self,
        rid: identity::RepoId,
        status: Option<cobs::query::PatchStatus>,
        skip: Option<usize>,
        take: Option<usize>,
    ) -> Result<cobs::PaginatedQuery<Vec<cobs::patch::Patch>>, Error> {
        let profile = self.profile();
        let cursor = skip.unwrap_or(0);
        let take = take.unwrap_or(20);
        let repo = profile.storage.repository(rid)?;
        let aliases = &profile.aliases();
        let cache = profile.patches(&repo, migrate::ignore)?;
        let patches = match status {
            None => cache.list()?.collect::<Vec<_>>(),
            Some(s) => cache.list_by_status(&s.into())?.collect::<Vec<_>>(),
        };
        let more = cursor + take < patches.len();

        let mut patches = patches
            .into_iter()
            .filter_map(|p| {
                p.map(|(id, patch)| cobs::patch::Patch::new(id, &patch, aliases))
                    .ok()
            })
            .skip(cursor)
            .take(take)
            .collect::<Vec<_>>();

        patches.sort_by_key(|b| std::cmp::Reverse(b.timestamp()));

        Ok::<_, Error>(cobs::PaginatedQuery {
            cursor,
            more,
            content: patches,
        })
    }

    fn get_patch(
        &self,
        rid: identity::RepoId,
        id: git::Oid,
    ) -> Result<Option<cobs::patch::Patch>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo, migrate::ignore)?;
        let patch = patches.get(&id.into())?;
        let aliases = &profile.aliases();
        let patches = patch.map(|patch| cobs::patch::Patch::new(id.into(), &patch, aliases));

        Ok::<_, Error>(patches)
    }

    fn revisions_by_patch(
        &self,
        rid: identity::RepoId,
        id: git::Oid,
    ) -> Result<Option<Vec<cobs::patch::Revision>>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo, migrate::ignore)?;
        let revisions = patches.get(&id.into())?.map(|patch| {
            let aliases = &profile.aliases();

            patch
                .revisions()
                .map(|(_, r)| cobs::patch::Revision::new(r.clone(), aliases))
                .collect::<Vec<_>>()
        });

        Ok::<_, Error>(revisions)
    }

    fn revision_by_id(
        &self,
        rid: identity::RepoId,
        id: git::Oid,
        revision_id: git::Oid,
    ) -> Result<Option<cobs::patch::Revision>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo, migrate::ignore)?;
        let revision = patches.get(&id.into())?.and_then(|patch| {
            let aliases = &profile.aliases();

            patch
                .revision(&revision_id.into())
                .map(|r| cobs::patch::Revision::new(r.clone(), aliases))
        });

        Ok::<_, Error>(revision)
    }
}

pub trait PatchesMut: Profile {
    fn edit_patch(
        &self,
        rid: identity::RepoId,
        cob_id: git::Oid,
        action: cobs::patch::Action,
        opts: cobs::CobOptions,
    ) -> Result<cobs::patch::Patch, Error> {
        let profile = self.profile();
        let mut node = Node::new(profile.socket());
        let repo = profile.storage.repository(rid)?;
        let signer = profile.signer()?;
        let aliases = profile.aliases();
        let mut patches = profile.patches_mut(&repo, migrate::ignore)?;
        let mut patch = patches.get_mut(&cob_id.into())?;

        match action {
            cobs::patch::Action::RevisionEdit {
                revision,
                description,
                embeds,
            } => {
                patch.edit_revision(
                    revision,
                    description,
                    embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::RevisionCommentRedact { revision, comment } => {
                patch.comment_redact(revision, comment, &signer)?;
            }
            cobs::patch::Action::ReviewCommentRedact { review, comment } => {
                patch.redact_review_comment(review, comment, &signer)?;
            }
            cobs::patch::Action::ReviewCommentReact {
                review,
                comment,
                reaction,
                active,
            } => {
                patch.react_review_comment(review, comment, reaction, active, &signer)?;
            }
            cobs::patch::Action::ReviewCommentResolve { review, comment } => {
                patch.resolve_review_comment(review, comment, &signer)?;
            }
            cobs::patch::Action::ReviewCommentUnresolve { review, comment } => {
                patch.unresolve_review_comment(review, comment, &signer)?;
            }
            cobs::patch::Action::Edit { title, target } => {
                patch.edit(title, target, &signer)?;
            }
            cobs::patch::Action::ReviewEdit {
                review,
                summary,
                verdict,
                labels,
            } => {
                patch.review_edit(review, verdict, summary, labels, &signer)?;
            }
            cobs::patch::Action::Review {
                revision,
                summary,
                verdict,
                labels,
            } => {
                patch.review(revision, verdict, summary, labels, &signer)?;
            }
            cobs::patch::Action::ReviewRedact { review } => {
                patch.redact_review(review, &signer)?;
            }
            cobs::patch::Action::ReviewComment {
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
                    embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::ReviewCommentEdit {
                review,
                comment,
                body,
                embeds,
            } => {
                patch.edit_review_comment(
                    review,
                    comment,
                    body,
                    embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::Lifecycle { state } => {
                patch.lifecycle(state, &signer)?;
            }
            cobs::patch::Action::Assign { assignees } => {
                patch.assign(assignees, &signer)?;
            }
            cobs::patch::Action::Label { labels } => {
                patch.label(labels, &signer)?;
            }
            cobs::patch::Action::RevisionReact {
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
            cobs::patch::Action::RevisionComment {
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
                    embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::RevisionCommentEdit {
                revision,
                comment,
                body,
                embeds,
            } => {
                patch.comment_edit(
                    revision,
                    comment,
                    body,
                    embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::RevisionCommentReact {
                revision,
                comment,
                reaction,
                active,
            } => {
                patch.comment_react(revision, comment, reaction, active, &signer)?;
            }
            cobs::patch::Action::RevisionRedact { revision } => {
                patch.redact(revision, &signer)?;
            }
            cobs::patch::Action::Merge { .. } => {
                unimplemented!("We don't support merging of patches through the desktop")
            }
            cobs::patch::Action::Revision { .. } => {
                unimplemented!("We don't support creating new revisions through the desktop")
            }
        }

        if opts.announce() {
            node.announce_refs(rid)?;
        }

        Ok::<_, Error>(cobs::patch::Patch::new(*patch.id(), &patch, &aliases))
    }

    /// Gets the draft review of the local user for a specific patch revision in a repository.
    ///
    /// This Tauri command is used to retrieve a patch review draft for the local user
    /// on a given patch revision from a repository.
    /// It looks up the repository using the provided repository ID (`rid`) and patch object ID (`cob_id`),
    /// and gets the patch review of the local user associated with a specific revision (`revision_id`), if it exists.
    fn get_draft_review(
        &self,
        rid: identity::RepoId,
        cob_id: git::Oid,
        revision_id: patch::RevisionId,
    ) -> Option<patch::Review> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid).ok()?;
        let signer = profile.signer().ok()?;
        let drafts = storage::git::cob::DraftStore::new(&repo, *signer.public_key());
        let patches = cob::patch::Cache::no_cache(&drafts).ok()?;

        let patch = patches.get(&cob_id.into()).ok()?;
        let revision = patch.and_then(|p| p.revision(&revision_id).cloned());
        let review = revision.and_then(|rev| rev.review_by(signer.public_key()).cloned());

        review
    }

    /// Edits a draft review for a specific patch revision in a repository.
    ///
    /// This Tauri command allows users to edit a draft review for a specific patch review.
    /// The draft is associated with the user (signer) and the provided patch revision within the repository.
    fn edit_draft_review(
        &self,
        rid: identity::RepoId,
        cob_id: git::Oid,
        edit: cobs::patch::ReviewEdit,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let signer = profile.signer()?;
        let drafts = storage::git::cob::DraftStore::new(&repo, *signer.public_key());

        let mut patches = cob::patch::Cache::no_cache(&drafts)?;
        let mut patch = patches.get_mut(&cob_id.into())?;
        patch.review_edit(
            edit.review_id,
            edit.verdict,
            edit.summary,
            edit.labels,
            &signer,
        )?;

        patches.write(&cob_id.into())?;

        Ok::<_, Error>(())
    }

    /// Creates a draft review for a specific patch revision.
    ///
    /// This Tauri command allows users to create a new draft review for a specific patch revision.
    /// The draft is associated with the user (signer) and the provided patch revision within the repository.
    fn create_draft_review(
        &self,
        rid: identity::RepoId,
        revision_id: cob::patch::RevisionId,
        cob_id: git::Oid,
        labels: Vec<cob::Label>,
    ) -> Result<patch::ReviewId, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let signer = profile.signer()?;
        let drafts = storage::git::cob::DraftStore::new(&repo, *signer.public_key());

        let mut patches = cob::patch::Cache::no_cache(&drafts)?;
        let mut patch = patches.get_mut(&cob_id.into())?;
        let revision = patch
            .revision(&revision_id)
            .ok_or_else(|| Error::WithHint {
                err: anyhow::anyhow!("patch revision not found"),
                hint: "Not able to find the specified patch revision.",
            })?;

        revision
            .review_by(signer.public_key())
            .ok_or(Error::WithHint {
                err: anyhow::anyhow!("duplicate patch review found"),
                hint: "Found an existing draft patch review on this patch revision and repo.",
            })?;

        let review_id = patch.review(
            revision.id(),
            Some(cob::patch::Verdict::Reject),
            None,
            labels,
            &signer,
        )?;

        patches.write(&cob_id.into())?;

        Ok::<_, Error>(review_id)
    }

    /// Creates a new review comment on a draft review for a specific patch.
    ///
    /// This Tauri command is used to add a comment to an existing draft review in a repository.
    /// It allows users to comment on a specific location in the code or leave general feedback
    /// on a review that belongs to a specific patch.
    fn create_draft_review_comment(
        &self,
        rid: identity::RepoId,
        cob_id: git::Oid,
        new: cobs::thread::CreateReviewComment,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let signer = profile.signer()?;
        let drafts = storage::git::cob::DraftStore::new(&repo, *signer.public_key());

        let mut patches = cob::patch::Cache::no_cache(&drafts)?;
        let mut patch = patches.get_mut(&cob_id.into())?;

        patch.transaction("Review comments", &signer, |tx| {
            tx.review_comment(
                new.review_id,
                new.body,
                new.location.map(|l| l.into()),
                new.reply_to,
                new.embeds.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
            )?;

            Ok(())
        })?;

        patches.write(&cob_id.into())?;

        Ok::<_, Error>(())
    }
}
