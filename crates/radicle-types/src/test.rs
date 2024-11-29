use std::path::Path;

use radicle::cob::migrate;
use radicle::crypto::ssh::Keystore;
use radicle::crypto::{KeyPair, Seed};
use radicle::node::{Features, Timestamp, UserAgent};
use radicle::profile::Home;
use radicle::Storage;
use radicle::{node, profile};

pub const TIMESTAMP: u64 = 1671125284;

/// Create a new profile.
pub fn profile(home: &Path, seed: [u8; 32]) -> radicle::Profile {
    let home = Home::new(home).unwrap();
    let keystore = Keystore::new(&home.keys());

    let keypair = KeyPair::from_seed(Seed::from(seed));
    let alias = node::Alias::new("seed");
    let storage = Storage::open(
        home.storage(),
        radicle::git::UserInfo {
            alias: alias.clone(),
            key: keypair.pk.into(),
        },
    )
    .unwrap();

    let mut db = home.policies_mut().unwrap();
    db.follow(&keypair.pk.into(), Some(&alias)).unwrap();

    let node_db = home.database_mut().unwrap();
    node_db
        .init(
            &keypair.pk.into(),
            Features::SEED,
            &alias,
            &UserAgent::default(),
            Timestamp::try_from(TIMESTAMP).unwrap(),
            [],
        )
        .unwrap();

    // Migrate COBs cache.
    let mut cobs = home.cobs_db_mut().unwrap();
    cobs.migrate(migrate::ignore).unwrap();

    radicle::storage::git::transport::local::register(storage.clone());
    keystore.store(keypair.clone(), "radicle", None).unwrap();

    radicle::Profile {
        home,
        storage,
        keystore,
        public_key: keypair.pk.into(),
        config: profile::Config::new(alias),
    }
}
