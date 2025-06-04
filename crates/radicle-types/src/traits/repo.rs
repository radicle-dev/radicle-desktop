use base64::Engine;
use radicle_surf as surf;
use serde::{Deserialize, Serialize};

use radicle::identity::{doc, Doc, DocAt};
use radicle::issue::cache::Issues as _;
use radicle::node::routing::Store;
use radicle::patch::cache::Patches as _;
use radicle::storage;
use radicle::storage::{ReadRepository, ReadStorage, RepositoryInfo};
use radicle::{git, identity, node};

use crate::cobs;
use crate::diff;
use crate::diff::Diff;
use crate::error::Error;
use crate::repo::{self, RepoCount};
use crate::syntax::{Highlighter, ToPretty};
use crate::traits::Profile;

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Show {
    Delegate,
    All,
    Contributor,
    Seeded,
    Private,
}

pub trait Repo: Profile {
    fn list_repos(&self, show: Show) -> Result<Vec<repo::RepoInfo>, Error> {
        let profile = self.profile();
        let storage = &profile.storage;
        let policies = profile.policies()?;
        let repos = storage.repositories()?;
        let mut entries = Vec::new();

        for RepositoryInfo { rid, doc, refs, .. } in repos {
            if refs.is_none() && show == Show::Contributor {
                continue;
            }

            if !policies.is_seeding(&rid)? && show == Show::Seeded {
                continue;
            }

            if !doc.is_private() && show == Show::Private {
                continue;
            }

            if !doc.delegates().contains(&profile.public_key.into()) && show == Show::Delegate {
                continue;
            }

            let repo = profile.storage.repository(rid)?;
            let repo_info = self.repo_info(&repo, &doc)?;

            entries.push(repo_info)
        }

        entries.sort_by_key(|repo_info| {
            repo_info
                .payloads
                .project
                .as_ref()
                .map(|p| p.name().to_lowercase())
        });

        Ok::<_, Error>(entries)
    }

    fn repo_count(&self) -> Result<repo::RepoCount, Error> {
        let profile = self.profile();
        let storage = &profile.storage;
        let policies = profile.policies()?;
        let repos = storage.repositories()?;
        let mut total = 0;
        let mut delegate = 0;
        let mut private = 0;
        let mut contributor = 0;
        let mut seeding = 0;

        for RepositoryInfo { rid, doc, refs, .. } in repos {
            total += 1;
            if policies.is_seeding(&rid)? {
                seeding += 1;
            }

            if doc.is_private() {
                private += 1;
            }

            if doc.delegates().contains(&profile.public_key.into()) {
                delegate += 1;
            }

            if refs.is_some() {
                contributor += 1;
            }
        }

        Ok::<_, Error>(RepoCount {
            total,
            contributor,
            seeding,
            private,
            delegate,
        })
    }

    fn repo_readme(
        &self,
        rid: identity::RepoId,
        sha: Option<git::Oid>,
    ) -> Result<Option<repo::Readme>, Error> {
        let profile = self.profile();
        let repo = radicle_surf::Repository::open(storage::git::paths::repository(
            &profile.storage,
            &rid,
        ))?;

        let paths = [
            "README",
            "README.md",
            "README.markdown",
            "README.txt",
            "README.rst",
            "README.org",
            "Readme.md",
        ];

        let oid = sha.map_or_else(|| repo.head(), Ok)?;
        for path in paths
            .iter()
            .map(ToString::to_string)
            .chain(paths.iter().map(|p| p.to_lowercase()))
        {
            if let Ok(blob) = repo.blob(oid, &path) {
                let content = match std::str::from_utf8(blob.content()) {
                    Ok(s) => s.to_owned(),
                    Err(_) => base64::engine::general_purpose::STANDARD.encode(blob.content()),
                };

                return Ok(Some(repo::Readme {
                    path,
                    content,
                    binary: blob.is_binary(),
                }));
            }
        }
        Ok(None)
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
    ) -> Result<diff::Stats, Error> {
        let profile = self.profile();
        let repo = radicle_surf::Repository::open(storage::git::paths::repository(
            &profile.storage,
            &rid,
        ))?;
        let base = repo.commit(base)?;
        let commit = repo.commit(head)?;
        let diff = repo.diff(base.id, commit.id)?;
        let stats = diff.stats();

        Ok::<_, Error>(diff::Stats::new(stats))
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
        let (_, head) = repo.head()?;
        let commit = repo.commit(head)?;
        let project = doc
            .payload()
            .get(&doc::PayloadId::project())
            .and_then(|payload| {
                let patches = profile.patches(repo).ok()?;
                let patches = patches.counts().ok()?;
                let issues = profile.issues(repo).ok()?;
                let issues = issues.counts().ok()?;

                let data: repo::ProjectPayloadData = (*payload).clone().try_into().ok()?;
                let meta = repo::ProjectPayloadMeta {
                    issues,
                    patches,
                    head,
                };

                Some(repo::ProjectPayload::new(data, meta))
            });

        Ok::<_, Error>(repo::RepoInfo {
            payloads: repo::SupportedPayloads { project },
            delegates,
            threshold: doc.threshold(),
            visibility: match doc.visibility().clone() {
                identity::Visibility::Public => repo::Visibility::Public,
                identity::Visibility::Private { allow } => repo::Visibility::Private {
                    allow: allow
                        .iter()
                        .map(|did| cobs::Author::new(did, &aliases))
                        .collect(),
                },
            },
            rid: repo.id,
            seeding,
            last_commit_timestamp: commit.time().seconds() * 1000,
        })
    }

    fn get_diff(
        &self,
        rid: identity::RepoId,
        options: cobs::diff::DiffOptions,
    ) -> Result<Diff, Error> {
        let unified = options.unified.unwrap_or(5);
        let highlight = options.highlight.unwrap_or(true);
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?.backend;
        let base = repo.find_commit(*options.base)?;
        let head = repo.find_commit(*options.head)?;

        let mut opts = git::raw::DiffOptions::new();
        opts.patience(true).minimal(true).context_lines(unified);

        let mut find_opts = git::raw::DiffFindOptions::new();
        find_opts.exact_match_only(true);
        find_opts.all(true);

        let left = base.tree()?;
        let right = head.tree()?;

        let mut diff = repo.diff_tree_to_tree(Some(&left), Some(&right), Some(&mut opts))?;
        diff.find_similar(Some(&mut find_opts))?;
        let diff = surf::diff::Diff::try_from(diff)?;

        if highlight {
            let mut hi = Highlighter::new();

            return Ok::<_, Error>(diff.pretty(&mut hi, &(), &repo));
        }

        Ok::<_, Error>(diff.into())
    }

    fn list_commits(
        &self,
        rid: identity::RepoId,
        base: String,
        head: String,
    ) -> Result<Vec<repo::Commit>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;

        let repo = surf::Repository::open(repo.path())?;
        let history = repo.history(&head)?;

        let commits = history
            .take_while(|c| {
                if let Ok(c) = c {
                    c.id.to_string() != base
                } else {
                    false
                }
            })
            .filter_map(|c| c.map(Into::into).ok())
            .collect();

        Ok(commits)
    }

    fn unseed(&self, rid: identity::RepoId) -> Result<(), Error> {
        let profile = self.profile();
        let mut node = radicle::Node::new(profile.socket());

        profile.unseed(rid, &mut node)?;

        Ok(())
    }

    fn seed(&self, rid: identity::RepoId) -> Result<(), Error> {
        let profile = self.profile();
        let mut node = radicle::Node::new(profile.socket());

        profile.seed(rid, node::policy::Scope::All, &mut node)?;

        Ok(())
    }

    fn seeded_not_replicated(&self) -> Result<Vec<identity::RepoId>, Error> {
        let profile = &self.profile();
        let storage = &profile.storage;
        let policies = profile.policies()?;
        let entries = policies
            .seed_policies()?
            .filter(|policy| !storage.contains(&policy.rid).unwrap_or(false))
            .map(|policy| policy.rid)
            .collect::<Vec<_>>();

        Ok(entries)
    }
}
