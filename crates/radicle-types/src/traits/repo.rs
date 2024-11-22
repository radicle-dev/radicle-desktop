use radicle::cob::migrate;
use radicle_surf as surf;
use serde::{Deserialize, Serialize};

use radicle::identity::{doc, Doc, DocAt};
use radicle::issue::cache::Issues as _;
use radicle::node::routing::Store;
use radicle::patch::cache::Patches as _;
use radicle::storage;
use radicle::storage::{ReadRepository, ReadStorage, RepositoryInfo};
use radicle::{git, identity};

use crate::cobs;
use crate::error::Error;
use crate::repo;
use crate::traits::Profile;

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Show {
    Delegate,
    All,
    Seeded,
}

pub trait Repo: Profile {
    fn list_repos(&self, show: Show) -> Result<Vec<repo::RepoInfo>, Error> {
        let profile = self.profile();
        let storage = &profile.storage;
        let policies = profile.policies()?;
        let repos = storage.repositories()?;
        let mut entries = Vec::new();

        for RepositoryInfo { rid, doc, refs, .. } in repos {
            if refs.is_none() && show == Show::All {
                continue;
            }

            if !policies.is_seeding(&rid)? && show == Show::Seeded {
                continue;
            }

            if !doc.delegates().contains(&profile.public_key.into()) && show == Show::Delegate {
                continue;
            }

            let repo = profile.storage.repository(rid)?;
            let repo_info = self.repo_info(&repo, &doc)?;

            entries.push(repo_info)
        }

        entries.sort_by_key(|repo::RepoInfo { rid, .. }| *rid);

        Ok::<_, Error>(entries)
    }

    fn repo_by_id(&self, rid: identity::RepoId) -> Result<repo::RepoInfo, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let DocAt { doc, .. } = repo.identity_doc()?;

        let repo_info = self.repo_info(&repo, &doc)?;

        Ok::<_, Error>(repo_info)
    }

    fn diff_stats(
        &self,
        rid: identity::RepoId,
        base: git::Oid,
        head: git::Oid,
    ) -> Result<cobs::Stats, Error> {
        let profile = self.profile();
        let repo = radicle_surf::Repository::open(storage::git::paths::repository(
            &profile.storage,
            &rid,
        ))?;
        let base = repo.commit(base)?;
        let commit = repo.commit(head)?;
        let diff = repo.diff(base.id, commit.id)?;
        let stats = diff.stats();

        Ok::<_, Error>(cobs::Stats::new(stats))
    }

    fn repo_info(
        &self,
        repo: &storage::git::Repository,
        doc: &Doc,
    ) -> Result<repo::RepoInfo, Error> {
        let profile = self.profile();
        let aliases = profile.aliases();
        let delegates = doc
            .delegates()
            .iter()
            .map(|did| cobs::Author::new(did, &aliases))
            .collect::<Vec<_>>();
        let db = profile.database()?;
        let seeding = db.count(&repo.id).unwrap_or_default();
        let project = doc
            .payload()
            .get(&doc::PayloadId::project())
            .and_then(|payload| {
                let (_, head) = repo.head().ok()?;
                let commit = repo.commit(head).ok()?;
                let patches = profile.patches(repo, migrate::ignore).ok()?;
                let patches = patches.counts().ok()?;
                let issues = profile.issues(repo, migrate::ignore).ok()?;
                let issues = issues.counts().ok()?;

                let data: repo::ProjectPayloadData = (*payload).clone().try_into().ok()?;
                let meta = repo::ProjectPayloadMeta {
                    issues,
                    patches,
                    head,
                    last_commit_timestamp: commit.time().seconds() * 1000,
                };

                Some(repo::ProjectPayload::new(data, meta))
            });

        Ok::<_, Error>(repo::RepoInfo {
            payloads: repo::SupportedPayloads { project },
            delegates,
            threshold: doc.threshold(),
            visibility: doc.visibility().clone().into(),
            rid: repo.id,
            seeding,
        })
    }

    fn get_diff(
        &self,
        rid: identity::RepoId,
        options: cobs::diff::Options,
    ) -> Result<surf::diff::Diff, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?.backend;
        let base = repo.find_commit(*options.base)?;
        let head = repo.find_commit(*options.head)?;

        let mut opts = git::raw::DiffOptions::new();
        opts.patience(true)
            .minimal(true)
            .context_lines(options.unified);

        let mut find_opts = git::raw::DiffFindOptions::new();
        find_opts.exact_match_only(true);
        find_opts.all(true);

        let left = base.tree()?;
        let right = head.tree()?;

        let mut diff = repo.diff_tree_to_tree(Some(&left), Some(&right), Some(&mut opts))?;
        diff.find_similar(Some(&mut find_opts))?;
        let diff = surf::diff::Diff::try_from(diff)?;

        Ok::<_, Error>(diff)
    }
}
