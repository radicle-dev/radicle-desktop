use crate::config::Config;

pub mod auth;
pub mod cobs;
pub mod issue;
pub mod patch;
pub mod repo;
pub mod thread;

pub trait Profile {
    fn profile(&self) -> radicle::Profile;

    fn config(&self) -> Config {
        let p = self.profile();

        Config {
            public_key: p.public_key,
            alias: p.config.node.alias.clone(),
            seeding_policy: p.config.node.seeding_policy,
        }
    }
}
