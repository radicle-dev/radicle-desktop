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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use std::str::FromStr;

    use radicle::crypto::test::signer::MockSigner;
    use radicle::crypto::Signer;
    use radicle::node::{config, Alias};

    use crate::config::Config;
    use crate::{test, AppState, Profile};

    #[test]
    fn config() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = test::profile(tmp.path(), [0xff; 32]);
        let signer = MockSigner::from_seed([0xff; 32]);
        let state = AppState { profile };

        assert_eq!(
            Profile::config(&state),
            Config {
                public_key: *signer.public_key(),
                alias: Alias::from_str("seed").unwrap(),
                seeding_policy: config::DefaultSeedingPolicy::Block,
            }
        )
    }
}
