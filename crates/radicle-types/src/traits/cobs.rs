use radicle::cob::migrate;
use radicle::cob::object::Storage;
use radicle::storage::refs::draft;
use radicle::storage::{self, ReadStorage};
use radicle::{cob, git, identity};

use crate::error::Error;
use crate::traits::Profile;

pub trait Cobs: Profile {
    fn activity_by_id(
        &self,
        rid: identity::RepoId,
        type_name: cob::TypeName,
        id: git::Oid,
    ) -> Result<Vec<crate::cobs::issue::Operation>, Error> {
        let profile = self.profile();
        let aliases = profile.aliases();
        let repo = profile.storage.repository(rid)?;
        let ops = cob::store::ops(&id.into(), &type_name, &repo).unwrap();
        let mut actions: Vec<crate::cobs::issue::Operation> = Vec::new();

        for op in ops.into_iter() {
            actions.extend(op.actions.iter().filter_map(
                |action: &Vec<u8>| -> Option<crate::cobs::issue::Operation> {
                    let action: crate::cobs::issue::Action = serde_json::from_slice(action).ok()?;

                    Some(crate::cobs::issue::Operation {
                        entry_id: op.id,
                        action,
                        author: crate::cobs::Author::new(&op.author.into(), &aliases),
                        timestamp: op.timestamp,
                    })
                },
            ))
        }

        Ok::<_, Error>(actions)
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

        let mut patches = profile.patches_mut(&repo, migrate::ignore)?;
        patches.write(&cob_id.into())?;

        storage::git::cob::DraftStore::new(&repo, *signer.public_key()).remove(
            signer.public_key(),
            &type_name,
            &cob_id.into(),
        )?;

        Ok::<_, Error>(())
    }
}
