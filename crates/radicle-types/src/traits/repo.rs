use base64::Engine;
use radicle::git::Oid;
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
use crate::repo;
use crate::source;
use crate::syntax::{highlighter, ToPretty};
use crate::traits::Profile;

pub const MAX_BLOB_SIZE: usize = 10_485_760;

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
            if refs.is_ok() && show == Show::Contributor {
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

    fn list_repos_summary(&self) -> Result<Vec<repo::RepoSummary>, Error> {
        let profile = self.profile();
        let storage = &profile.storage;

        let entries = storage
            .repositories()?
            .into_iter()
            .filter_map(|RepositoryInfo { rid, doc, .. }| {
                doc.payload()
                    .get(&doc::PayloadId::project())
                    .and_then(|payload| repo::ProjectPayloadData::try_from((*payload).clone()).ok())
                    .map(|data| (rid, data))
            })
            .map(|(rid, data)| {
                let summary = repo::RepoSummary {
                    rid,
                    name: data.name,
                };

                (summary.name.to_lowercase(), summary)
            })
            .collect::<BTreeMap<_, _>>();

        Ok::<_, Error>(entries.values().collect())
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

            if refs.is_ok() {
                contributor += 1;
            }
        }

        Ok::<_, Error>(repo::RepoCount {
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

        let oid = sha.map_or_else(|| repo.head().map(|oid| Oid::from(*oid)), Ok)?;

        for path in paths
            .iter()
            .map(ToString::to_string)
            .chain(paths.iter().map(|p| p.to_lowercase()))
        {
            if let Ok(blob) = repo.blob(crate::oid::into_surf(oid), &path) {
                if blob.size() > MAX_BLOB_SIZE {
                    return Err(Error::FileTooLarge(blob.size()));
                }

                let content = match std::str::from_utf8(blob.content()) {
                    Ok(s) => s.to_owned(),
                    Err(_) => base64::engine::general_purpose::STANDARD.encode(blob.content()),
                };

                return Ok(Some(repo::Readme {
                    id: blob.object_id(),
                    commit: blob.commit().clone().into(),
                    mime_type: "text/plain".to_owned(),
                    path,
                    content,
                    binary: blob.is_binary(),
                }));
            }
        }
        Ok(None)
    }

    fn repo_tree(
        &self,
        rid: identity::RepoId,
        path: std::path::PathBuf,
    ) -> Result<source::tree::Tree, Error> {
        let profile = self.profile();
        let repo = radicle_surf::Repository::open(radicle::storage::git::paths::repository(
            &profile.storage,
            &rid,
        ))?;
        let head = repo.head()?;
        let tree = repo.tree(head, &path)?;
        Ok(source::tree::Tree::from_surf(tree, &path))
    }

    fn repo_blob(
        &self,
        rid: identity::RepoId,
        path: std::path::PathBuf,
    ) -> Result<source::blob::Blob, Error> {
        let profile = self.profile();
        let repo = radicle_surf::Repository::open(radicle::storage::git::paths::repository(
            &profile.storage,
            &rid,
        ))?;
        let head = repo.head()?;

        repo.blob(head, &path).map(Into::into).map_err(Error::from)
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
        let base = repo.commit(crate::oid::into_surf(base))?;
        let commit = repo.commit(crate::oid::into_surf(head))?;
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
        let base = repo.find_commit(options.base.into())?;
        let head = repo.find_commit(options.head.into())?;

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
            return Ok::<_, Error>(diff.pretty(highlighter(), &(), &repo));
        }

        Ok::<_, Error>(diff.into())
    }

    fn get_commit_diff(
        &self,
        rid: identity::RepoId,
        sha: git::Oid,
        unified: Option<u32>,
        highlight: Option<bool>,
    ) -> Result<Diff, Error> {
        let unified = unified.unwrap_or(5);
        let highlight = highlight.unwrap_or(true);
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?.backend;
        let head = repo.find_commit(sha.into())?;

        let mut opts = git::raw::DiffOptions::new();
        opts.patience(true).minimal(true).context_lines(unified);

        let mut find_opts = git::raw::DiffFindOptions::new();
        find_opts.exact_match_only(true);
        find_opts.all(true);

        let left = head
            .parents()
            .next()
            .map(|parent| parent.tree())
            .transpose()?;
        let right = head.tree()?;

        let mut diff = repo.diff_tree_to_tree(left.as_ref(), Some(&right), Some(&mut opts))?;
        diff.find_similar(Some(&mut find_opts))?;
        let diff = surf::diff::Diff::try_from(diff)?;

        if highlight {
            return Ok::<_, Error>(diff.pretty(highlighter(), &(), &repo));
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

    fn list_repo_commits(
        &self,
        rid: identity::RepoId,
        head: Option<git::Oid>,
        skip: Option<usize>,
        take: Option<usize>,
    ) -> Result<crate::cobs::PaginatedQuery<Vec<repo::Commit>>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;

        let repo = surf::Repository::open(repo.path())?;
        let head = match head {
            Some(head) => crate::oid::into_surf(head),
            None => repo.head()?,
        };
        let commits = repo.history(head)?;
        let cursor = skip.unwrap_or(0);

        match take {
            None => {
                let content: Vec<repo::Commit> =
                    commits.filter_map(|c| c.map(Into::into).ok()).collect();

                Ok(crate::cobs::PaginatedQuery {
                    cursor: 0,
                    more: false,
                    content,
                })
            }
            Some(take) => {
                let content: Vec<repo::Commit> = commits
                    .filter_map(|c| c.map(Into::into).ok())
                    .skip(cursor)
                    .take(take + 1)
                    .collect();
                let more = content.len() > take;
                let content = if more {
                    content[..take].to_vec()
                } else {
                    content
                };

                Ok(crate::cobs::PaginatedQuery {
                    cursor,
                    more,
                    content,
                })
            }
        }
    }

    fn repo_commit_count(&self, rid: identity::RepoId, head: git::Oid) -> Result<usize, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;

        let repo = surf::Repository::open(repo.path())?;
        let count = repo.history(crate::oid::into_surf(head))?.count();

        Ok(count)
    }

    fn repo_commit(&self, rid: identity::RepoId, sha: git::Oid) -> Result<repo::Commit, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;

        let repo = surf::Repository::open(repo.path())?;
        let commit = repo.commit(crate::oid::into_surf(sha))?;

        Ok(commit.into())
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
            .filter_map(Result::ok)
            .filter(|policy| !storage.contains(&policy.rid).unwrap_or(false))
            .map(|policy| policy.rid)
            .collect::<Vec<_>>();

        Ok(entries)
    }
}
