use radicle::cob::identity;
use radicle::storage::ReadStorage;

use crate::cobs::Author;
use crate::error::Error;
use crate::identity as types;
use crate::traits::Profile;

pub trait Identity: Profile {
    /// The repository's identity document together with its full revision
    /// history. Identity revisions are rare — a handful over a repo's life —
    /// so the history is returned whole rather than paginated.
    fn identity_by_repo(&self, rid: radicle::identity::RepoId) -> Result<types::Identity, Error> {
        let profile = self.profile();
        let aliases = profile.aliases();
        let repo = profile.storage.repository(rid)?;
        let identity = identity::Identity::load(&repo)?;

        // Revisions are keyed by id so a revision can find its parent's
        // document and report what it changed.
        let docs = identity
            .revisions()
            .map(|revision| (revision.id, revision.doc.clone()))
            .collect::<std::collections::HashMap<_, _>>();

        let mut revisions = identity
            .revisions()
            .map(|revision| {
                let parent = revision.parent.and_then(|id| docs.get(&id));

                types::Revision {
                    id: revision.id,
                    blob: revision.blob,
                    title: revision.title.to_string(),
                    description: revision.description.clone(),
                    state: revision.state.into(),
                    author: Author::new(&revision.author.id, &aliases),
                    timestamp: revision.timestamp.as_secs() * 1000,
                    parent: revision.parent,
                    accepted: revision
                        .accepted()
                        .map(|did| Author::new(&did, &aliases))
                        .collect(),
                    rejected: revision
                        .rejected()
                        .map(|did| Author::new(&did, &aliases))
                        .collect(),
                    // Quorum is a majority of the delegate set of the document
                    // in force before this revision — the parent's, or for the
                    // root revision its own. Signatures from anyone who was not
                    // a delegate at the time do not count towards it.
                    quorum: {
                        let governing = parent.unwrap_or(&revision.doc);
                        governing.is_majority(
                            revision
                                .accepted()
                                .filter(|did| governing.is_delegate(did))
                                .count(),
                        )
                    },
                    changes: types::changes(parent, &revision.doc, &aliases),
                    doc: types::Doc::new(&revision.doc, &aliases),
                }
            })
            .collect::<Vec<_>>();

        // Newest first, matching how the app lists patches and issues.
        revisions.reverse();

        Ok(types::Identity {
            rid,
            current: identity.current().id,
            doc: types::Doc::new(identity.doc(), &aliases),
            revisions,
        })
    }
}
