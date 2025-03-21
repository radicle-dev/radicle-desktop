use radicle::{cob, git, identity};
use serde::de::DeserializeOwned;

use crate::domain::repo::models::cobs;
use crate::error::Error;

pub trait RepoActivity {
    fn activity_by_id<A: DeserializeOwned, B: cobs::FromRadicleAction<A>>(
        &self,
        rid: identity::RepoId,
        type_name: &cob::TypeName,
        id: git::Oid,
    ) -> Result<Vec<cobs::Operation<B>>, Error>;
}
