use radicle::identity::doc::{Delegates, DocAt};
use radicle::storage::{ReadRepository, ReadStorage};
use radicle::{git, identity};

use radicle_artifact::{Artifact, ReleaseId, Releases as ArtifactStore, cache_db_path};

use crate::cobs;
use crate::error::Error;
use crate::traits::Profile;

/// Whether an artifact was redacted by its own author or by a delegate.
fn redacted_by_trusted(artifact: &Artifact, delegates: &Delegates) -> bool {
    artifact
        .redactions()
        .keys()
        .any(|did| did == artifact.author() || delegates.contains(did))
}

/// Whether every artifact of a release was redacted by a trusted party. A
/// release without artifacts is not redacted; it has nothing to redact.
fn release_redacted(release: &radicle_artifact::Release, delegates: &Delegates) -> bool {
    let artifacts = release.artifacts();

    !artifacts.is_empty()
        && artifacts
            .values()
            .all(|artifact| redacted_by_trusted(artifact, delegates))
}

/// How far the caller widened the default, delegate-scoped release view.
#[derive(Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReleaseFilter {
    /// Include releases and artifacts authored by non-delegates.
    pub all_authors: bool,
    /// Include artifacts redacted by their author or a delegate.
    pub show_redacted: bool,
}

impl ReleaseFilter {
    /// Whether an artifact is shown under this view: authored by a delegate
    /// (or `all_authors`), and not redacted by its author or a delegate (or
    /// `show_redacted`).
    fn show_artifact(&self, artifact: &Artifact, delegates: &Delegates) -> bool {
        (self.all_authors || delegates.contains(artifact.author()))
            && (self.show_redacted || !redacted_by_trusted(artifact, delegates))
    }

    /// Whether a release is shown under this view: created by a delegate (or
    /// `all_authors`), and not left with all of its artifacts redacted by a
    /// trusted party (or `show_redacted`).
    fn show_release(&self, release: &radicle_artifact::Release, delegates: &Delegates) -> bool {
        (self.all_authors || delegates.contains(release.creator()))
            && (self.show_redacted || !release_redacted(release, delegates))
    }
}

pub trait Releases: Profile {
    /// List a repository's releases, newest first.
    ///
    /// Scoped to releases created by a delegate and artifacts authored by a
    /// delegate (hiding those redacted by a trusted party) unless widened with
    /// `filter`. A release whose artifacts were all redacted is hidden with
    /// them. Without `take` the full list is returned and `skip` is ignored.
    fn list_releases(
        &self,
        rid: identity::RepoId,
        filter: Option<ReleaseFilter>,
        skip: Option<usize>,
        take: Option<usize>,
    ) -> Result<cobs::PaginatedQuery<Vec<cobs::release::Release>>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let DocAt { doc, .. } = repo.identity_doc()?;
        let delegates = doc.delegates();
        let aliases = profile.aliases();
        let filter = filter.unwrap_or_default();

        // Read through the SQLite cache; it self-warms on read and is shared
        // with other release reads on this node.
        let store = ArtifactStore::open_cached(&repo, cache_db_path(profile.cobs()))?;
        let mut releases = store
            .all()?
            .into_iter()
            .filter_map(|entry| {
                let (id, release) = entry.ok()?;
                filter
                    .show_release(&release, delegates)
                    .then_some((id, release))
            })
            .collect::<Vec<_>>();
        releases.sort_by_key(|(_, release)| std::cmp::Reverse(release.timestamp()));

        let summary = |(id, release): (radicle::cob::ObjectId, radicle_artifact::Release)| {
            let artifacts = release
                .artifacts()
                .iter()
                .filter(|(_, artifact)| filter.show_artifact(artifact, delegates))
                .map(|(cid, artifact)| cobs::release::Artifact::new(cid, artifact, &aliases))
                .collect::<Vec<_>>();

            cobs::release::Release::new(ReleaseId::from(id), &release, &repo, &aliases, artifacts)
        };

        match take {
            None => Ok(cobs::PaginatedQuery {
                cursor: 0,
                more: false,
                content: releases.into_iter().map(summary).collect::<Vec<_>>(),
            }),
            Some(take) => {
                let cursor = skip.unwrap_or(0);
                let mut content = releases
                    .into_iter()
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
                .map(|(cid, artifact)| cobs::release::Artifact::new(cid, artifact, &aliases))
                .collect::<Vec<_>>();

            cobs::release::Release::new(ReleaseId::from(id), &release, &repo, &aliases, artifacts)
        }))
    }

    /// Number of releases in a repository, counted from a git ref walk without
    /// materializing any of them.
    fn release_count(&self, rid: identity::RepoId) -> Result<usize, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let store = ArtifactStore::open_cached(&repo, cache_db_path(profile.cobs()))?;

        Ok(store.count()?)
    }
}
