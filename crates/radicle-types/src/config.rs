use radicle::crypto::PublicKey;
use serde::Serialize;
use ts_rs::TS;

use radicle::node::config::DefaultSeedingPolicy;
use radicle::node::Alias;

/// Service configuration.
#[derive(Debug, TS, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "config/")]
pub struct Config {
    /// Node Public Key in NID format.
    #[ts(as = "String")]
    pub public_key: PublicKey,
    /// Node alias.
    #[ts(as = "String")]
    pub alias: Alias,
    /// Default seeding policy.
    #[serde(default)]
    #[ts(type = "{ default: 'allow', scope: 'followed' | 'all' } | { default: 'block' }")]
    pub seeding_policy: DefaultSeedingPolicy,
}

impl Config {
    pub fn get(profile: &radicle::Profile) -> Self {
        Self {
            public_key: profile.public_key,
            seeding_policy: profile.config.node.seeding_policy,
            alias: profile.config.node.alias.clone(),
        }
    }
}
