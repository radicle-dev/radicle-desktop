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
use crate::diff::Diff;
use crate::error::Error;
use crate::repo;
use crate::syntax::{Highlighter, ToPretty};
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

        entries.sort_by(|a, b| b.last_commit_timestamp.cmp(&a.last_commit_timestamp));

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
            visibility: doc.visibility().clone().into(),
            rid: repo.id,
            seeding,
            last_commit_timestamp: commit.time().seconds() * 1000,
        })
    }

    fn get_diff(&self, rid: identity::RepoId, options: cobs::diff::Options) -> Result<Diff, Error> {
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
            let mut hi = Highlighter::default();

            return Ok::<_, Error>(diff.pretty(&mut hi, &(), &repo));
        }

        Ok::<_, Error>(diff.into())
    }

    fn list_commits(
        &self,
        rid: identity::RepoId,
        parent: Option<String>,
        skip: Option<usize>,
        take: Option<usize>,
    ) -> Result<cobs::PaginatedQuery<Vec<repo::Commit>>, Error> {
        let profile = self.profile();
        let cursor = skip.unwrap_or(0);
        let take = take.unwrap_or(20);
        let repo = profile.storage.repository(rid)?;

        let sha = match parent {
            Some(commit) => commit,
            None => repo.head()?.1.to_string(),
        };

        let repo = surf::Repository::open(repo.path())?;
        let history = repo.history(&sha)?;

        let mut commits = history
            .filter_map(|c| c.map(Into::into).ok())
            .skip(cursor)
            .take(take + 1); // Take one extra item to check if there's more.

        let paginated_commits: Vec<_> = commits.by_ref().take(take).collect();
        let more = commits.next().is_some();

        Ok::<_, Error>(cobs::PaginatedQuery {
            cursor,
            more,
            content: paginated_commits.to_vec(),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use std::str::FromStr;

    use radicle::crypto::test::signer::MockSigner;
    use radicle::{git, test};
    use radicle_surf::Author;

    use crate::cobs;
    use crate::repo;
    use crate::traits::repo::Repo;
    use crate::AppState;

    #[test]
    fn list_commits_pagination() {
        let signer = MockSigner::from_seed([0xff; 32]);
        let tempdir = tempfile::tempdir().unwrap();
        let profile = crate::test::profile(tempdir.path(), [0xff; 32]);
        let (rid, _, _, _) =
            test::fixtures::project(tempdir.path().join("original"), &profile.storage, &signer)
                .unwrap();
        let state = AppState { profile };
        let commits = Repo::list_commits(&state, rid, None, None, Some(1)).unwrap();

        assert_eq!(
            commits,
            cobs::PaginatedQuery {
                cursor: 0,
                more: true,
                content: vec![repo::Commit {
                    id: git::Oid::from_str("f2de534b5e81d7c6e2dcaf58c3dd91573c0a0354").unwrap(),
                    author: Author {
                        name: "anonymous".to_string(),
                        email: "anonymous@radicle.xyz".to_string(),
                        time: radicle::git::raw::Time::new(1514817556, 0).into(),
                    },
                    committer: Author {
                        name: "anonymous".to_string(),
                        email: "anonymous@radicle.xyz".to_string(),
                        time: radicle::git::raw::Time::new(1514817556, 0).into(),
                    },
                    message: "Second commit".to_string(),
                    summary: "Second commit".to_string(),
                    parents: vec![
                        git::Oid::from_str("08c788dd1be6315de09e3fe09b5b1b7a2b8711d9").unwrap()
                    ],
                }],
            }
        );

        let commits = Repo::list_commits(&state, rid, None, Some(1), None).unwrap();

        assert_eq!(
            commits,
            cobs::PaginatedQuery {
                cursor: 1,
                more: false,
                content: vec![repo::Commit {
                    id: git::Oid::from_str("08c788dd1be6315de09e3fe09b5b1b7a2b8711d9").unwrap(),
                    author: Author {
                        name: "anonymous".to_string(),
                        email: "anonymous@radicle.xyz".to_string(),
                        time: radicle::git::raw::Time::new(1514817556, 0).into(),
                    },
                    committer: Author {
                        name: "anonymous".to_string(),
                        email: "anonymous@radicle.xyz".to_string(),
                        time: radicle::git::raw::Time::new(1514817556, 0).into(),
                    },
                    message: "Initial commit".to_string(),
                    summary: "Initial commit".to_string(),
                    parents: vec![],
                }],
            }
        );
    }

    #[test]
    fn list_commits_with_head() {
        let signer = MockSigner::from_seed([0xff; 32]);
        let tempdir = tempfile::tempdir().unwrap();
        let profile = crate::test::profile(tempdir.path(), [0xff; 32]);
        let (rid, _, _, _) =
            test::fixtures::project(tempdir.path().join("original"), &profile.storage, &signer)
                .unwrap();
        let state = AppState { profile };
        let commits = Repo::list_commits(
            &state,
            rid,
            Some("08c788dd1be6315de09e3fe09b5b1b7a2b8711d9".to_string()),
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            commits,
            cobs::PaginatedQuery {
                cursor: 0,
                more: false,
                content: vec![repo::Commit {
                    id: git::Oid::from_str("08c788dd1be6315de09e3fe09b5b1b7a2b8711d9").unwrap(),
                    author: Author {
                        name: "anonymous".to_string(),
                        email: "anonymous@radicle.xyz".to_string(),
                        time: radicle::git::raw::Time::new(1514817556, 0).into(),
                    },
                    committer: Author {
                        name: "anonymous".to_string(),
                        email: "anonymous@radicle.xyz".to_string(),
                        time: radicle::git::raw::Time::new(1514817556, 0).into(),
                    },
                    message: "Initial commit".to_string(),
                    summary: "Initial commit".to_string(),
                    parents: vec![],
                }],
            }
        );
    }
}
