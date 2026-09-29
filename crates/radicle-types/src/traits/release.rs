use std::collections::BTreeSet;

use radicle::identity::Did;
use radicle::storage::ReadStorage;
use radicle::{git, identity};

use radicle_artifact::trust::Trust;
use radicle_artifact::{Filters, ReleaseId, Releases as ArtifactStore, cache_db_path};

use crate::cobs;
use crate::error::Error;
use crate::traits::Profile;

/// How far the caller widened the default, delegate-scoped release view.
#[derive(Clone, Copy, Default, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
#[ts(export_to = "cob/release/")]
pub struct ReleaseFilter {
    /// Include releases and artifacts authored by non-delegates.
    pub all_authors: bool,
    /// Include artifacts redacted by their author or a delegate.
    pub show_redacted: bool,
}

impl ReleaseFilter {
    /// The crate's visibility rules for this view. No local user is trusted,
    /// so the default view matches the `delegate` bucket of `counts()`.
    fn filters(self, delegates: &BTreeSet<Did>) -> Filters<'_> {
        Filters {
            trust: Trust {
                delegates,
                local: None,
                all_authors: self.all_authors,
            },
            redacted: self.show_redacted,
        }
    }
}

pub trait Releases: Profile {
    /// List a repository's releases, newest first.
    ///
    /// Scoped to releases created by a delegate and artifacts authored by a
    /// delegate (hiding those redacted by a trusted party) unless widened with
    /// `filter`. A release with no delegate artifact shows every author's. A release whose artifacts were all redacted is hidden with
    /// them; a release with no artifacts is shown. This is the view
    /// `rad-artifact list` gives. Without `take` the full list is returned and
    /// `skip` is ignored.
    fn list_releases(
        &self,
        rid: identity::RepoId,
        filter: Option<ReleaseFilter>,
        skip: Option<usize>,
        take: Option<usize>,
    ) -> Result<cobs::PaginatedQuery<Vec<cobs::release::Release>>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let aliases = profile.aliases();
        let filter = filter.unwrap_or_default();

        // Read through the SQLite cache; it self-warms on read and is shared
        // with other release reads on this node. `list` is sorted newest
        // first and lazy, so a page reads only the rows up to its end.
        let store = ArtifactStore::open_cached(&repo, cache_db_path(profile.cobs()))?;
        let filters = filter.filters(store.delegates());
        let releases = store
            .list()?
            .filter_map(Result::ok)
            .filter(|(_, release)| filters.shows_release(release));

        // With no delegate artifact left to show, fall back to every author,
        // as the release page does, so a teaser never counts fewer artifacts
        // than its page lists.
        let fallback = ReleaseFilter {
            all_authors: true,
            ..filter
        };
        let fallback = fallback.filters(store.delegates());
        let summary = |(id, release): (radicle::cob::ObjectId, radicle_artifact::Release)| {
            let shown = |filters: &Filters| {
                release
                    .artifacts()
                    .iter()
                    .filter(|(_, artifact)| filters.shows_artifact(artifact))
                    .map(|(cid, artifact)| {
                        cobs::release::Artifact::new(cid, artifact, store.delegates(), &aliases)
                    })
                    .collect::<Vec<_>>()
            };
            let mut artifacts = shown(&filters);
            if artifacts.is_empty() && !filter.all_authors {
                artifacts = shown(&fallback);
            }

            cobs::release::Release::new(ReleaseId::from(id), &release, &repo, &aliases, artifacts)
        };

        match take {
            None => Ok(cobs::PaginatedQuery {
                cursor: 0,
                more: false,
                content: releases.map(summary).collect::<Vec<_>>(),
            }),
            Some(take) => {
                let cursor = skip.unwrap_or(0);
                let mut content = releases
                    .skip(cursor)
                    .take(take + 1)
                    .map(summary)
                    .collect::<Vec<_>>();
                let more = content.len() > take;
                content.truncate(take);

                Ok(cobs::PaginatedQuery {
                    cursor,
                    more,
                    content,
                })
            }
        }
    }

    /// Get a single release by id, with all of its artifacts. Filtering is left
    /// to the caller so a release reached by a direct link is never empty.
    fn release_by_id(
        &self,
        rid: identity::RepoId,
        id: git::Oid,
    ) -> Result<Option<cobs::release::Release>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let aliases = profile.aliases();

        let store = ArtifactStore::open_cached(&repo, cache_db_path(profile.cobs()))?;
        let release = store.get(&ReleaseId::from(id))?;

        Ok(release.map(|release| {
            let artifacts = release
                .artifacts()
                .iter()
                .map(|(cid, artifact)| {
                    cobs::release::Artifact::new(cid, artifact, store.delegates(), &aliases)
                })
                .collect::<Vec<_>>();

            cobs::release::Release::new(ReleaseId::from(id), &release, &repo, &aliases, artifacts)
        }))
    }

    /// Number of releases in a repository, bucketed the same way the default
    /// list filters them.
    fn release_counts(&self, rid: identity::RepoId) -> Result<cobs::release::ReleaseCounts, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let store = ArtifactStore::open_cached(&repo, cache_db_path(profile.cobs()))?;

        Ok(store.counts()?.into())
    }
}
