use std::collections::BTreeMap;

use base64::Engine;
use radicle_surf as surf;
use serde::{Deserialize, Serialize};

use radicle::identity::{Doc, DocAt, doc};
use radicle::issue::cache::Issues as _;
use radicle::node::AliasStore;
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
use crate::syntax::{ToPretty, highlighter};
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

/// A repo counts as one the local node contributes to whenever it has
/// local signed refs, even if those refs are at an outdated feature
/// level pending node-side migration.
fn is_contributor(refs: &storage::SignedRefsInfo) -> bool {
    matches!(
        refs,
        storage::SignedRefsInfo::Some(_) | storage::SignedRefsInfo::NeedsMigration,
    )
}

/// Resolve a `(peer, revision)` pair to a commit OID. A named revision is
/// looked up under the peer's namespaced `refs/heads` and `refs/tags` when a
/// peer is given, otherwise under the canonical top-level refs. With no
/// revision, a peer resolves to its head of the project's default branch, and
/// no peer resolves to the canonical head. Raw commit OIDs are passed through
/// the handlers' `sha`/`head` argument instead of this function.
fn resolve_revision(
    repo: &storage::git::Repository,
    peer: Option<node::NodeId>,
    revision: Option<String>,
) -> Result<git::Oid, Error> {
    let lookup = |refname: String| -> Option<git::Oid> {
        let r = repo.backend.find_reference(&refname).ok()?;
        let commit = r.peel_to_commit().ok()?;
        Some(commit.id().into())
    };

    match peer {
        Some(peer) => {
            let name = match revision {
                Some(rev) => rev,
                None => {
                    let DocAt { doc, .. } = repo.identity_doc()?;
                    doc.project()
                        .map_err(|e| Error::RevisionNotFound(e.to_string()))?
                        .default_branch()
                        .to_string()
                }
            };
            ["refs/heads", "refs/tags"]
                .iter()
                .find_map(|prefix| lookup(format!("refs/namespaces/{peer}/{prefix}/{name}")))
                .ok_or_else(|| Error::RevisionNotFound(format!("{name} under peer {peer}")))
        }
        None => match revision {
            Some(name) => ["refs/heads", "refs/tags"]
                .iter()
                .find_map(|prefix| lookup(format!("{prefix}/{name}")))
                .ok_or(Error::RevisionNotFound(name)),
            None => {
                let (_, head) = repo.head()?;
                Ok(head)
            }
        },
    }
}

/// Collect canonical branches and tags as declared by the repository's
/// identity document. The canonical-refs rules (the `xyz.radicle.crefs`
/// payload, or a synthesized default covering the project's default branch)
/// define which ref patterns are canonical; each pattern is globbed against
/// the storage repo's resolved top-level refs (per-peer refs live under
/// `refs/namespaces/`). Only refs under `refs/heads` and `refs/tags` are
/// kept, and refs that cannot be peeled to a commit are skipped.
fn canonical_refs(repo: &storage::git::Repository) -> Result<repo::Canonical, Error> {
    let mut canonical = repo::Canonical::default();

    let DocAt { doc, .. } = repo.identity_doc()?;
    let crefs = doc
        .canonical_refs()
        .map_err(storage::RepositoryError::from)?;
    let rules = git::canonical::rules::RawRules::from(crefs.rules().clone());

    for (pattern, _) in rules.iter() {
        for r in repo.backend.references_glob(pattern.as_str())? {
            let r = r?;
            let Some(name) = r.name() else { continue };
            let Some(oid) = r.target() else { continue };

            if let Some(short) = name.strip_prefix("refs/tags/") {
                let Some(tag) = resolve_tag(repo, oid) else {
                    continue;
                };
                canonical.tags.insert(short.to_owned(), tag);
            } else if let Some(short) = name.strip_prefix("refs/heads/") {
                let Ok(commit) = repo
                    .backend
                    .find_object(oid, None)
                    .and_then(|obj| obj.peel_to_commit())
                else {
                    continue;
                };
                canonical
                    .branches
                    .insert(short.to_owned(), commit.id().into());
            }
        }
    }

    Ok(canonical)
}

/// Resolve a ref OID to a [`repo::Tag`]. For annotated tags uses tagger time;
/// for lightweight tags uses the target commit's time. Returns `None` if the
/// OID cannot be peeled to a commit.
fn resolve_tag(repo: &storage::git::Repository, oid: git::raw::Oid) -> Option<repo::Tag> {
    if let Ok(tag) = repo.backend.find_tag(oid) {
        let target_oid = tag.target_id();
        let commit = repo.backend.find_commit(target_oid).ok()?;
        let tagger = tag.tagger().map(|t| repo::Tagger {
            name: t.name().unwrap_or_default().to_owned(),
            email: t.email().unwrap_or_default().to_owned(),
            timestamp: t.when().seconds(),
        });
        let timestamp = tagger
            .as_ref()
            .map(|t| t.timestamp)
            .unwrap_or_else(|| commit.time().seconds());
        return Some(repo::Tag {
            oid: commit.id().into(),
            timestamp,
            tagger,
            message: tag.message().map(str::to_owned),
        });
    }
    let commit = repo.backend.find_commit(oid).ok()?;
    Some(repo::Tag {
        oid: commit.id().into(),
        timestamp: commit.time().seconds(),
        tagger: None,
        message: None,
    })
}

/// Partition a remote's refs into short-name branch and tag maps. Refs that
/// cannot be peeled to a commit, are not qualified, or are not under
/// `refs/heads` or `refs/tags` are skipped.
fn partition_refs(
    refs: &storage::refs::Refs,
    repo: &storage::git::Repository,
) -> (BTreeMap<String, git::Oid>, BTreeMap<String, repo::Tag>) {
    let mut branches = BTreeMap::new();
    let mut tags = BTreeMap::new();

    for (refname, oid) in refs.iter() {
        let Some(qualified) = refname.qualified() else {
            continue;
        };

        let (_, category, first, rest) = qualified.non_empty_components();
        let name = std::iter::once(first)
            .chain(rest)
            .collect::<git::fmt::RefString>()
            .to_string();

        match category.as_str() {
            "heads" => {
                let Ok(commit) = repo
                    .backend
                    .find_object((*oid).into(), None)
                    .and_then(|obj| obj.peel_to_commit())
                else {
                    continue;
                };
                branches.insert(name, commit.id().into());
            }
            "tags" => {
                let Some(tag) = resolve_tag(repo, (*oid).into()) else {
                    continue;
                };
                tags.insert(name, tag);
            }
            _ => {}
        }
    }

    (branches, tags)
}

pub trait Repo: Profile {
    fn list_repos(&self, show: Show) -> Result<Vec<repo::RepoInfo>, Error> {
        let profile = self.profile();
        let storage = &profile.storage;
        let policies = profile.policies()?;
        let repos = storage.repositories()?;
        let mut entries = Vec::new();

        for RepositoryInfo { rid, doc, refs, .. } in repos {
            if !is_contributor(&refs) && show == Show::Contributor {
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
        let repos = storage.repositories()?;
        let mut entries = Vec::new();

        for RepositoryInfo { rid, doc, .. } in repos {
            let Some(data) = doc
                .payload()
                .get(&doc::PayloadId::project())
                .and_then(|payload| repo::ProjectPayloadData::try_from((*payload).clone()).ok())
            else {
                continue;
            };
            entries.push(repo::RepoSummary {
                rid,
                name: data.name,
            });
        }

        entries.sort_by_key(|r| r.name.to_lowercase());

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

            if is_contributor(&refs) {
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
        peer: Option<node::NodeId>,
        revision: Option<String>,
    ) -> Result<Option<repo::Readme>, Error> {
        let profile = self.profile();
        let storage_repo = profile.storage.repository(rid)?;
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

        let oid = match sha {
            Some(sha) => sha,
            None => resolve_revision(&storage_repo, peer, revision)?,
        };

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
        sha: Option<git::Oid>,
        peer: Option<node::NodeId>,
        revision: Option<String>,
    ) -> Result<source::tree::Tree, Error> {
        let profile = self.profile();
        let storage_repo = profile.storage.repository(rid)?;
        let repo = radicle_surf::Repository::open(radicle::storage::git::paths::repository(
            &profile.storage,
            &rid,
        ))?;
        let oid = match sha {
            Some(sha) => sha,
            None => resolve_revision(&storage_repo, peer, revision)?,
        };
        let tree = repo.tree(crate::oid::into_surf(oid), &path)?;
        Ok(source::tree::Tree::from_surf(tree, &path))
    }

    fn repo_blob(
        &self,
        rid: identity::RepoId,
        path: std::path::PathBuf,
        sha: Option<git::Oid>,
    ) -> Result<source::blob::Blob, Error> {
        let profile = self.profile();
        let repo = radicle_surf::Repository::open(radicle::storage::git::paths::repository(
            &profile.storage,
            &rid,
        ))?;
        let oid = sha.map_or_else(|| repo.head(), |sha| Ok(crate::oid::into_surf(sha)))?;

        repo.blob(oid, &path).map(Into::into).map_err(Error::from)
    }

    fn list_repo_refs(&self, rid: identity::RepoId) -> Result<repo::RepoRefs, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let DocAt { doc, .. } = repo.identity_doc()?;
        let delegates = doc.delegates();
        let aliases = profile.aliases();

        let mut remotes = Vec::new();
        for entry in repo.remotes()? {
            let (id, remote) = entry?;
            let (branches, tags) = partition_refs(&remote.refs, &repo);
            remotes.push(repo::Remote {
                id,
                alias: aliases.alias(&id),
                delegate: delegates.contains(&id.into()),
                branches,
                tags,
            });
        }

        let canonical = canonical_refs(&repo).unwrap_or_default();

        Ok(repo::RepoRefs { canonical, remotes })
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
        peer: Option<node::NodeId>,
        revision: Option<String>,
        skip: Option<usize>,
        take: Option<usize>,
    ) -> Result<crate::cobs::PaginatedQuery<Vec<repo::Commit>>, Error> {
        let profile = self.profile();
        let storage_repo = profile.storage.repository(rid)?;

        let oid = match head {
            Some(head) => head,
            None => resolve_revision(&storage_repo, peer, revision)?,
        };

        let repo = surf::Repository::open(storage_repo.path())?;
        let head = crate::oid::into_surf(oid);
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

        // Walk the commit graph directly instead of through radicle-surf's
        // History iterator, which materializes and parses a full Commit for
        // every node just to discard it. Counting only needs the reachable
        // OIDs, and no ordering, so the walk is unsorted. The walk itself is
        // not a verification step: trust comes from the signed `head` tip, and
        // git's content-addressed DAG keeps its ancestry tamper-evident.
        let mut revwalk = repo.backend.revwalk()?;
        revwalk.set_sorting(git2::Sort::NONE)?;
        revwalk.push(head.into())?;

        Ok(revwalk.count())
    }

    fn repo_commit(
        &self,
        rid: identity::RepoId,
        sha: Option<git::Oid>,
        peer: Option<node::NodeId>,
        revision: Option<String>,
    ) -> Result<repo::Commit, Error> {
        let profile = self.profile();
        let storage_repo = profile.storage.repository(rid)?;

        let oid = match sha {
            Some(sha) => sha,
            None => resolve_revision(&storage_repo, peer, revision)?,
        };

        let repo = surf::Repository::open(storage_repo.path())?;
        let commit = repo.commit(crate::oid::into_surf(oid))?;

        Ok(commit.into())
    }

    /// List the repo's tags. Used by the New Release form so users can
    /// pick an annotated tag (which records its OID alongside the commit
    /// on the artifact COB) instead of typing a raw commit OID.
    fn list_tags(&self, rid: identity::RepoId) -> Result<Vec<repo::TagRef>, Error> {
        use radicle_surf::{Glob, Tag};

        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let surf_repo = surf::Repository::open(repo.path())?;

        let mut tags = Vec::new();
        for tag in surf_repo.tags(&Glob::all_tags())? {
            let Ok(tag) = tag else { continue };
            let entry = match tag {
                Tag::Light { id, name } => repo::TagRef {
                    name: name.to_string(),
                    oid: id.to_string(),
                    commit: id.to_string(),
                    annotated: false,
                    message: None,
                },
                Tag::Annotated {
                    id,
                    target,
                    name,
                    message,
                    ..
                } => repo::TagRef {
                    name: name.to_string(),
                    oid: id.to_string(),
                    commit: target.to_string(),
                    annotated: true,
                    message,
                },
            };
            tags.push(entry);
        }
        // Newest tag first by name (semver-ish sort) — surf already returns
        // them sorted; keep the natural order as-is.
        Ok(tags)
    }

    fn unseed(&self, rid: identity::RepoId) -> Result<(), Error> {
        let profile = self.profile();
        let mut node = radicle::Node::new(profile.home().socket_from_env());

        profile.unseed(rid, &mut node)?;

        Ok(())
    }

    fn seed(&self, rid: identity::RepoId) -> Result<(), Error> {
        let profile = self.profile();
        let mut node = radicle::Node::new(profile.home().socket_from_env());

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
