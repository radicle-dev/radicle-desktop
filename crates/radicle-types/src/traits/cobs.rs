use radicle::cob::object::Storage;
use radicle::storage::refs::draft;
use radicle::storage::{self, ReadStorage};
use radicle::{cob, git, identity};
use serde::de::DeserializeOwned;

use crate::error::Error;
use crate::traits::Profile;

pub trait Cobs: Profile {
    #[allow(clippy::unnecessary_filter_map)]
    fn activity_by_id<A: DeserializeOwned>(
        &self,
        rid: identity::RepoId,
        type_name: &cob::TypeName,
        id: git::Oid,
    ) -> Result<Vec<crate::cobs::Operation<A>>, Error> {
        let profile = self.profile();
        let aliases = profile.aliases();
        let repo = profile.storage.repository(rid)?;
        let iter = cob::store::ops(&id.into(), type_name, &repo)?;
        let ops = iter
            .into_iter()
            .filter_map(|op| {
                let actions = op
                    .actions
                    .iter()
                    .filter_map(|a| serde_json::from_slice(a).ok())
                    .collect::<Vec<_>>();

                Some(crate::cobs::Operation {
                    id: op.id,
                    actions,
                    author: crate::cobs::Author::new(&op.author.into(), &aliases),
                    timestamp: op.timestamp,
                })
            })
            .collect::<Vec<_>>();

        Ok::<_, Error>(ops)
    }

    fn publish_draft(
        &self,
        rid: identity::RepoId,
        cob_id: git::Oid,
        type_name: cob::TypeName,
    ) -> Result<(), Error> {
        let profile = self.profile();
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;
        let draft_oid = repo.backend.refname_to_id(&draft::cob(
            signer.public_key(),
            &type_name,
            &cob_id.into(),
        ))?;
        repo.update(
            signer.public_key(),
            &type_name,
            &cob_id.into(),
            &draft_oid.into(),
        )?;

        let mut patches = profile.patches_mut(&repo)?;
        patches.write(&cob_id.into())?;

        storage::git::cob::DraftStore::new(&repo, *signer.public_key()).remove(
            signer.public_key(),
            &type_name,
            &cob_id.into(),
        )?;

        Ok::<_, Error>(())
    }
}
