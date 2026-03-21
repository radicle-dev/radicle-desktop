pub(crate) fn from_surf(oid: radicle_surf::Oid) -> radicle::git::Oid {
    radicle::git::Oid::from(radicle::git::raw::Oid::from(oid))
}

pub(crate) fn into_surf(oid: radicle::git::Oid) -> radicle_surf::Oid {
    radicle_surf::Oid::from(radicle::git::raw::Oid::from(oid))
}
