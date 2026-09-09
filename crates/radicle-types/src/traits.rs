use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use radicle::node;
use radicle::node::{Alias, AliasStore, NodeId};

use crate::cobs::AliasSuggestion;
use crate::config::Config;

/// Upper bound on the number of suggestions returned by
/// [`Profile::search_aliases`], so a broad query cannot flood the frontend.
const ALIAS_SEARCH_LIMIT: usize = 50;

pub mod cobs;
pub mod issue;
pub mod job;
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
            public_explorer: p.config.public_explorer.clone(),
            preferred_seeds: p.config.preferred_seeds.clone(),
        }
    }

    fn alias(&self, nid: NodeId) -> Option<radicle::node::Alias> {
        let p = self.profile();
        let aliases = p.aliases();

        aliases.alias(&nid)
    }

    /// Find nodes whose alias matches `query`, for autocompleting mentions.
    ///
    /// Candidates come from the nodes the local node follows, from the gossiped
    /// address book, and from the local node itself. An empty `query` returns
    /// the followed nodes only, since the alias stores cannot be enumerated and
    /// an empty alias is not a valid `Alias`.
    ///
    /// Aliases are self-declared, so a popular name can match dozens of
    /// unrelated nodes. Followed nodes are ranked ahead of gossiped ones and
    /// every result carries the flags the frontend needs to label them.
    fn search_aliases(&self, query: Option<String>) -> Vec<AliasSuggestion> {
        let p = self.profile();
        let query = query.unwrap_or_default();
        let query = query.trim();
        let policies = p.policies().ok();

        // Nodes we follow are the highest-signal candidates, and the only
        // source that can be listed without a search term.
        let mut candidates: BTreeMap<NodeId, Option<Alias>> = BTreeMap::new();
        let mut followed = BTreeSet::new();
        // The `following` table holds blocked nodes alongside followed ones, so
        // they have to be filtered out of every source, not just this one.
        let mut blocked = BTreeSet::new();
        if let Some(policies) = policies.as_ref()
            && let Ok(follow_policies) = policies.follow_policies()
        {
            for policy in follow_policies.flatten() {
                match policy.policy {
                    node::policy::Policy::Allow => {
                        followed.insert(policy.nid);
                        candidates.insert(policy.nid, policy.alias);
                    }
                    node::policy::Policy::Block => {
                        blocked.insert(policy.nid);
                    }
                }
            }
        }

        // The local node, so that mentioning yourself works.
        candidates.insert(p.public_key, Some(p.config.node.alias.clone()));

        // `Aliases::reverse_lookup` merges its two stores with `BTreeMap::extend`,
        // which drops node ids when both stores hold the same alias. Query the
        // stores separately so every matching node id survives.
        if let Ok(alias) = Alias::from_str(query) {
            if let Some(policies) = policies.as_ref() {
                extend_candidates(&mut candidates, policies.reverse_lookup(&alias));
            }
            if let Ok(db) = p.database() {
                extend_candidates(&mut candidates, db.reverse_lookup(&alias));
            }
        }

        let needle = query.to_lowercase();
        let mut matches = candidates
            .into_iter()
            .filter(|(nid, _)| !blocked.contains(nid))
            .filter_map(|(nid, alias)| {
                let rank = rank_alias(alias.as_ref(), &needle, followed.contains(&nid))?;

                Some((rank, alias.clone(), nid))
            })
            .collect::<Vec<_>>();

        matches.sort_by(|(a_rank, a_alias, a_nid), (b_rank, b_alias, b_nid)| {
            a_rank
                .cmp(b_rank)
                .then_with(|| a_alias.cmp(b_alias))
                .then_with(|| a_nid.cmp(b_nid))
        });
        matches.truncate(ALIAS_SEARCH_LIMIT);

        matches
            .into_iter()
            .map(|(_, alias, nid)| AliasSuggestion {
                did: nid.into(),
                alias,
                followed: followed.contains(&nid),
                is_self: nid == p.public_key,
            })
            .collect()
    }
}

/// Merge the result of an [`AliasStore::reverse_lookup`] into `candidates`.
fn extend_candidates(
    candidates: &mut BTreeMap<NodeId, Option<Alias>>,
    found: BTreeMap<Alias, BTreeSet<NodeId>>,
) {
    for (alias, nids) in found {
        for nid in nids {
            candidates.insert(nid, Some(alias.clone()));
        }
    }
}

/// Score how well `alias` matches `needle`, lower being better. Returns `None`
/// when the candidate should be filtered out entirely.
///
/// An exact match sorts before a prefix match, which sorts before a match
/// anywhere in the alias. Followed nodes outrank merely gossiped ones, since
/// they are nodes the user chose to keep track of.
fn rank_alias(alias: Option<&Alias>, needle: &str, followed: bool) -> Option<u8> {
    let position = match alias {
        // A node with no alias is only worth suggesting when nothing was typed.
        None if needle.is_empty() => 3,
        None => return None,
        Some(alias) => {
            let alias = alias.to_string().to_lowercase();

            if alias == needle {
                0
            } else if alias.starts_with(needle) {
                1
            } else if alias.contains(needle) {
                2
            } else {
                return None;
            }
        }
    };

    Some(position * 2 + u8::from(!followed))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use std::str::FromStr;

    use radicle::crypto::{Seed, Signer, SigningKey};
    use radicle::node::{Alias, config};

    use crate::config::Config;
    use crate::{AppState, Profile, test};

    #[test]
    fn config() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = test::profile(tmp.path(), [0xff; 32]);
        let signer = SigningKey::from_seed(Seed::new([0xff; 32]));
        let state = AppState { profile };

        assert_eq!(
            Profile::config(&state),
            Config {
                public_key: *signer.public_key(),
                alias: Alias::from_str("seed").unwrap(),
                seeding_policy: config::DefaultSeedingPolicy::Block,
                public_explorer: state.profile.config.public_explorer.clone(),
                preferred_seeds: state.profile.config.preferred_seeds.clone(),
            }
        )
    }
}
