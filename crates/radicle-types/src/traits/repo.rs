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

#[cfg(windows)]
use std::os::windows::process::CommandExt;
// See <https://learn.microsoft.com/windows/win32/procthread/process-creation-flags#flags>.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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

/// Tally `git diff --numstat` between two commits into diff stats. Returns
/// `None` if git is unavailable or its output can't be parsed, so the caller
/// can fall back to the (slower) radicle-surf diff.
fn numstat(repo_dir: &std::path::Path, base: git::Oid, head: git::Oid) -> Option<diff::Stats> {
    let mut command = std::process::Command::new("git");
    command
        .current_dir(repo_dir)
        // Porcelain `git diff` honours user configuration (diff.renames,
        // diff.algorithm, external diff drivers, …), which would make the
        // reported stats machine-dependent and diverge from the surf
        // fallback. Pointing both config scopes at /dev/null pins the
        // output to git's defaults.
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .arg("diff")
        .arg("--numstat")
        .arg(base.to_string())
        .arg(head.to_string());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let output = command
        .output()
        .ok()
        .filter(|output| output.status.success())?;

    let mut stats = diff::Stats {
        files_changed: 0,
        insertions: 0,
        deletions: 0,
    };
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        // Each line is "<added>\t<deleted>\t<path>"; binary files report "-".
        let mut cols = line.split('\t');
        let added = cols.next()?;
        let deleted = cols.next()?;
        if cols.next().is_none() {
            continue;
        }
        stats.files_changed += 1;
        stats.insertions += added.parse::<usize>().unwrap_or(0);
        stats.deletions += deleted.parse::<usize>().unwrap_or(0);
    }
    Some(stats)
}

/// Resolve the most recent commit that modified `path`, reachable from `head`.
///
/// Fast path: `git rev-list` walks the history using the commit-graph (when
/// present), skipping the per-commit tree diff that libgit2 performs for a
/// pathspec walk. On large histories (e.g. the Linux kernel) the libgit2 walk
/// is seconds-to-minutes for a file last touched long ago, while this is
/// near-instant. Not a verification step: trust comes from the signed tip and
/// git's content-addressed DAG; the commit-graph is a local derived index over
/// those same objects. Falls back to the libgit2 walk if git is unavailable.
fn last_path_commit(
    surf_repo: &surf::Repository,
    repo_path: &std::path::Path,
    head: git::Oid,
    path: &std::path::Path,
) -> Result<repo::Commit, Error> {
    let mut command = std::process::Command::new("git");
    command
        .current_dir(repo_path)
        .arg("rev-list")
        .arg("-1")
        .arg(head.to_string())
        .arg("--")
        // `:(literal)` disables pathspec glob matching so file names
        // containing `[`, `*` or `?` (e.g. `src/pages/[id].ts`) are looked
        // up verbatim instead of being treated as wildcard patterns.
        .arg(format!(":(literal){}", path.display()));
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let fast = command
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<git::Oid>()
                .ok()
        });

    let commit = match fast {
        Some(oid) => surf_repo.commit(crate::oid::into_surf(oid))?,
        None => surf_repo
            .last_commit(&path, crate::oid::into_surf(head))?
            .ok_or_else(|| git2::Error::from_str("no commit found for path"))?,
    };

    Ok(commit.into())
}

/// The `git2::Diff` between `base` and `head` with the app's canonical
/// options (patience, minimal, exact-match rename detection). With `base`
/// unset the diff is taken against `head`'s first parent, or the empty tree
/// for a root commit. `show_binary` additionally embeds full binary deltas
/// so serialized patch text stays `git apply`-able.
fn tree_diff<'a>(
    repo: &'a git2::Repository,
    base: Option<git::Oid>,
    head: git::Oid,
    unified: u32,
    show_binary: bool,
) -> Result<git2::Diff<'a>, Error> {
    let head = repo.find_commit(head.into())?;
    let left = match base {
        Some(base) => Some(repo.find_commit(base.into())?.tree()?),
        None => head
            .parents()
            .next()
            .map(|parent| parent.tree())
            .transpose()?,
    };
    let right = head.tree()?;

    let mut opts = git::raw::DiffOptions::new();
    opts.patience(true)
        .minimal(true)
        .context_lines(unified)
        .show_binary(show_binary);

    let mut find_opts = git::raw::DiffFindOptions::new();
    find_opts.exact_match_only(true);
    find_opts.all(true);

    let mut diff = repo.diff_tree_to_tree(left.as_ref(), Some(&right), Some(&mut opts))?;
    diff.find_similar(Some(&mut find_opts))?;

    Ok(diff)
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
            let Ok(name) = r.name() else { continue };
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
            message: tag.message().ok().flatten().map(str::to_owned),
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
        let repo_path = storage::git::paths::repository(&profile.storage, &rid);
        let surf_repo = radicle_surf::Repository::open(&repo_path)?;

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
        let tree = storage_repo.backend.find_commit(oid.into())?.tree()?;

        for path in paths
            .iter()
            .map(ToString::to_string)
            .chain(paths.iter().map(|p| p.to_lowercase()))
        {
            let Ok(entry) = tree.get_path(std::path::Path::new(&path)) else {
                continue;
            };
            let Ok(blob) = entry
                .to_object(&storage_repo.backend)
                .and_then(|object| object.peel_to_blob())
            else {
                continue;
            };

            if blob.size() > MAX_BLOB_SIZE {
                return Err(Error::FileTooLarge(blob.size()));
            }

            let content = match std::str::from_utf8(blob.content()) {
                Ok(s) => s.to_owned(),
                Err(_) => base64::engine::general_purpose::STANDARD.encode(blob.content()),
            };
            // A failed last-commit lookup skips this candidate instead of
            // failing the whole call, so the repo home still renders (at
            // worst without a README), matching the pre-rewrite behaviour.
            let Ok(last_commit) =
                last_path_commit(&surf_repo, &repo_path, oid, std::path::Path::new(&path))
            else {
                continue;
            };

            return Ok(Some(repo::Readme {
                id: radicle_surf::Oid::from(blob.id()),
                commit: last_commit,
                mime_type: "text/plain".to_owned(),
                path,
                content,
                binary: blob.is_binary(),
            }));
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
        let storage_repo = profile.storage.repository(rid)?;
        let repo_path = storage::git::paths::repository(&profile.storage, &rid);
        let surf_repo = radicle_surf::Repository::open(&repo_path)?;

        let oid = match sha {
            Some(sha) => sha,
            None => crate::oid::from_surf(surf_repo.head()?),
        };

        // Resolve the blob via a direct tree lookup. `surf::Repository::blob`
        // additionally walks history to find the last commit that touched the
        // path, which we do separately (and cheaply) below.
        let commit = storage_repo.backend.find_commit(oid.into())?;
        let entry = commit.tree()?.get_path(&path)?;
        let blob = entry
            .to_object(&storage_repo.backend)?
            .into_blob()
            .map_err(|_| git2::Error::from_str("path does not point to a blob"))?;

        let last_commit = last_path_commit(&surf_repo, &repo_path, oid, &path)?;

        Ok(source::blob::Blob::new(
            blob.id().into(),
            blob.is_binary(),
            last_commit,
            blob.content(),
        ))
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

        // Fast path: `git diff --numstat` opens the repo and tallies per-file
        // line counts far faster than radicle-surf's full-content diff. List
        // views request stats for every patch row, and each surf diff re-opens
        // the whole repo (seconds in aggregate on a large repo). Falls back to
        // the surf diff if the git binary is unavailable or output can't parse.
        let repo_path = storage::git::paths::repository(&profile.storage, &rid);
        if let Some(stats) = numstat(&repo_path, base, head) {
            return Ok(stats);
        }

        let repo = radicle_surf::Repository::open(&repo_path)?;
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
        let diff = tree_diff(&repo, Some(options.base), options.head, unified, false)?;
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
        let diff = tree_diff(&repo, None, sha, unified, false)?;
        let diff = surf::diff::Diff::try_from(diff)?;

        if highlight {
            return Ok::<_, Error>(diff.pretty(highlighter(), &(), &repo));
        }

        Ok::<_, Error>(diff.into())
    }

    /// Serialize a diff as `git diff`-format patch text via libgit2, built
    /// with the same options as `get_diff`/`get_commit_diff` so the text
    /// matches the rendered diff. When `base` is unset the diff is taken
    /// against `head`'s first parent (or the empty tree for a root commit),
    /// mirroring `get_commit_diff`. When `path` is set, output is limited to
    /// that file's delta (matching either side of a rename).
    fn get_diff_text(
        &self,
        rid: identity::RepoId,
        base: Option<git::Oid>,
        head: git::Oid,
        unified: Option<u32>,
        path: Option<String>,
    ) -> Result<String, Error> {
        let unified = unified.unwrap_or(5);
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?.backend;
        let diff = tree_diff(&repo, base, head, unified, true)?;

        let path = path.map(std::path::PathBuf::from);
        let mut buf = Vec::new();
        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            if let Some(path) = path.as_deref()
                && delta.new_file().path() != Some(path)
                && delta.old_file().path() != Some(path)
            {
                return true;
            }
            // Content lines carry their origin marker ('+', '-', ' ')
            // separately from the text; header and EOF-marker lines already
            // include their full text.
            match line.origin() {
                '+' | '-' | ' ' => buf.push(line.origin() as u8),
                _ => {}
            }
            buf.extend_from_slice(line.content());
            true
        })?;

        Ok(String::from_utf8_lossy(&buf).into_owned())
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

        // Fast path: `git rev-list --count` uses the pack bitmap / commit-graph
        // (when present) for a near-instant count, whereas libgit2's revwalk
        // ignores those indexes and walks every commit (seconds on large
        // histories). Not a verification step: trust comes from the signed
        // `head` tip and git's content-addressed DAG keeps ancestry
        // tamper-evident; the bitmap/commit-graph are local derived indexes
        // over those same objects, not a new trust input. Falls back to the
        // walk below if the git binary is unavailable or errors.
        let mut command = std::process::Command::new("git");
        command.current_dir(repo.backend.path()).args([
            "rev-list",
            "--count",
            "--use-bitmap-index",
            &head.to_string(),
        ]);
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);
        let count = command
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse::<usize>()
                    .ok()
            });
        if let Some(count) = count {
            return Ok(count);
        }

        // Fallback: unsorted libgit2 walk (no Commit materialization, no
        // ordering — counting only needs the reachable OIDs).
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
