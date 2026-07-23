// `radicle_surf::Oid` and `radicle::git::Oid` are both aliases for
// `radicle_oid::Oid`, so these conversions are identities. They are kept as a
// single boundary in case the two crates diverge on their Oid type again.
pub(crate) fn from_surf(oid: radicle_surf::Oid) -> radicle::git::Oid {
    oid
}

pub(crate) fn into_surf(oid: radicle::git::Oid) -> radicle_surf::Oid {
    oid
}
