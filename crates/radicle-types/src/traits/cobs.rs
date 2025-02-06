use radicle::storage::ReadStorage;
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
}
