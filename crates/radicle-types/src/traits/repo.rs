use std::collections::{BTreeMap, BTreeSet};

use base64::Engine;
use radicle_surf as surf;
use serde::{Deserialize, Serialize};

use radicle::identity::{Doc, DocAt, doc};
use radicle::issue::cache::Issues as _;
use radicle::node::AliasStore;
use radicle::node::routing::Store;
use radicle::patch::cache::Patches as _;
use radicle::storage;
use radicle::storage::{ReadRepository, ReadStorage, RepositoryInfo, WriteStorage};
use radicle::{git, identity, node};

use crate::cobs;
use crate::diff;
use crate::diff::Diff;
use crate::error::Error;
use crate::repo;
use crate::source;
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
        Some(oid) => surf_repo.commit(oid)?,
        None => surf_repo
            .last_commit(&path, head)?
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

/// The `sshsig` namespace git signs commits under. A signature made for any
/// other namespace is not a statement about this commit and must not be
/// accepted for it.
const GIT_SIGNATURE_NAMESPACE: &str = "git";

/// Check the `gpgsig` header on `oid` against the bytes it covers.
///
/// Returns `None` when the commit carries no such header, otherwise the outcome
/// paired with the signing key when one could be attributed. This answers only
/// "does this signature verify, and which key made it" — deciding whether that
/// key is authorized for the repository is the caller's job.
fn verify_signature(
    backend: &git::raw::Repository,
    oid: git::Oid,
) -> Option<(repo::SignatureStatus, Option<node::NodeId>)> {
    // libgit2 splits the object into the `gpgsig` value and the exact bytes that
    // were signed, which is the commit object with that header removed.
    // Reconstructing the payload by re-serializing a parsed commit would not be
    // byte-identical and so would fail to verify.
    let (signature, payload) = backend.extract_signature(&oid.into(), None).ok()?;

    let Ok(signature) = ssh_key::SshSig::from_pem(&signature[..]) else {
        // Not an `sshsig` container: PGP, or something unparseable.
        return Some((repo::SignatureStatus::Unsupported, None));
    };
    // A signature made under a different namespace is a statement about
    // something other than this commit, so it must not count for it. Radicle's
    // own collaborative-object signatures live under the `radicle` namespace and
    // cover a commit id rather than a commit payload; they are rejected here.
    if signature.namespace() != GIT_SIGNATURE_NAMESPACE {
        return Some((repo::SignatureStatus::Unsupported, None));
    }
    let ssh_key::public::KeyData::Ed25519(key) = signature.public_key() else {
        // A Radicle identity is always Ed25519, so any other algorithm is a
        // signature we cannot attribute to a node.
        return Some((repo::SignatureStatus::Unsupported, None));
    };

    // `verify` re-derives the signed blob from the namespace, hash algorithm and
    // payload before checking it against the key carried in the signature.
    let verified = ssh_key::PublicKey::from(signature.public_key().clone())
        .verify(GIT_SIGNATURE_NAMESPACE, &payload[..], &signature)
        .is_ok();

    // Git does the signing, with whatever `user.signingKey` points at. When an
    // author points that at their Radicle key, the Ed25519 key in the signature
    // is also their node ID, so it resolves to a DID with no lookup in between.
    // Nothing enforces the convention: any other Ed25519 key yields a DID that
    // corresponds to no known node.
    let signer = node::NodeId::from(key.0);

    Some(if verified {
        (repo::SignatureStatus::Verified, Some(signer))
    } else {
        // The key is reported even when the check fails, so the UI can say whose
        // key failed rather than just that something did.
        (repo::SignatureStatus::Invalid, Some(signer))
    })
}

/// Per-repository context for resolving a signing key to a Radicle identity.
///
/// The delegate set and remote set are read once and reused across a batch of
/// commits, so verifying a page of history costs one identity lookup, not one
/// per commit.
struct Signers<'a, A> {
    backend: &'a git::raw::Repository,
    /// Nodes the local node knows of independently of any alias: those it
    /// follows, and those gossip says seed this repository.
    seen: BTreeSet<node::NodeId>,
    delegates: Option<doc::Delegates>,
    /// Every DID that has been a delegate under some accepted identity
    /// revision, including the current ones.
    historical: BTreeSet<identity::Did>,
    remotes: BTreeSet<node::NodeId>,
    aliases: &'a A,
}

impl<'a, A: AliasStore> Signers<'a, A> {
    /// Signature reporting is decoration on a commit listing, so a repository
    /// whose identity or remotes cannot be read still lists its commits — with
    /// the signature checked and the signer named, but no claim about whether
    /// that signer is a delegate.
    fn new(
        repo: &'a storage::git::Repository,
        rid: identity::RepoId,
        profile: &radicle::Profile,
        aliases: &'a A,
    ) -> Self {
        // An alias only says we have a *name* for a node, which is narrower
        // than knowing the node exists: a node followed without one, or known
        // from routing gossip, has no alias but is not a stranger either.
        let mut seen = BTreeSet::new();
        if let Ok(policies) = profile.policies()
            && let Ok(followed) = policies.follow_policies()
        {
            seen.extend(followed.flatten().map(|policy| policy.nid));
        }
        if let Ok(routing) = profile.routing()
            && let Ok(seeds) = routing.get(&rid)
        {
            seen.extend(seeds);
        }

        // Only accepted revisions count. `Identity::revisions` also yields
        // revisions still `Active` — proposed, but never adopted by a quorum —
        // and a single delegate can propose a revision naming anyone. Counting
        // those would make delegate history self-assertable.
        let historical = repo
            .identity()
            .map(|identity| {
                identity
                    .revisions()
                    .filter(|revision| revision.is_accepted())
                    .flat_map(|revision| revision.doc.delegates().iter().copied())
                    .collect()
            })
            .unwrap_or_default();

        Self {
            backend: &repo.backend,
            seen,
            delegates: repo
                .identity_doc()
                .ok()
                .map(|DocAt { doc, .. }| doc.delegates().clone()),
            historical,
            remotes: repo
                .remote_ids()
                .map(|ids| ids.filter_map(|id| id.ok()).collect())
                .unwrap_or_default(),
            aliases,
        }
    }

    /// Verify the `gpgsig` header on `oid`, if it has one.
    ///
    /// Returns `None` for an unsigned commit. Note that a returned
    /// [`repo::CommitSignature`] answers "who holds the key that signed these
    /// bytes", not "is this person authorized" — the `delegate` and `remote`
    /// flags are what the caller needs to turn a verified key into trust.
    fn verify(&self, oid: git::Oid) -> Option<repo::CommitSignature> {
        let (status, signer) = verify_signature(self.backend, oid)?;
        let did = signer.map(identity::Did::from);
        let delegate = matches!(
            (&self.delegates, did),
            (Some(delegates), Some(did)) if delegates.contains(&did)
        );

        let former_delegate = !delegate && did.is_some_and(|did| self.historical.contains(&did));
        let remote = signer.is_some_and(|signer| self.remotes.contains(&signer));
        let author = did.map(|did| cobs::Author::new(&did, self.aliases));

        Some(repo::CommitSignature {
            status,
            known: delegate
                || former_delegate
                || remote
                || author.as_ref().is_some_and(|a| a.alias().is_some())
                || signer.is_some_and(|signer| self.seen.contains(&signer)),
            fingerprint: signer.map(|signer| radicle::crypto::ssh::fmt::fingerprint(&signer)),
            signer: author,
            delegate,
            former_delegate,
            remote,
        })
    }
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
        let policies = profile.policies()?;
        let repos = storage.repositories()?;
        let mut entries = Vec::new();

        for RepositoryInfo { rid, doc, .. } in repos {
            // `rad unseed` only drops the seeding policy; the repository stays
            // in storage. This list backs the sidebar, which is about what you
            // seed, so an unseeded repo has to leave it even though its files
            // are still on disk — otherwise unseeding looks like it did nothing.
            if !policies.is_seeding(&rid)? {
                continue;
            }

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
        let tree = repo.tree(oid, &path)?;
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
            None => surf_repo.head()?,
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

    /// A manifest of the files a diff touches and their stats — not the lines,
    /// which the app gets from `get_diff_text`. Nothing here depends on how much
    /// context a hunk carries, so the diff is taken with none: it is thrown away
    /// either way, and computing it is work.
    fn get_diff(
        &self,
        rid: identity::RepoId,
        options: cobs::diff::DiffOptions,
    ) -> Result<Diff, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?.backend;
        let diff = tree_diff(&repo, Some(options.base), options.head, 0, false)?;
        let diff = surf::diff::Diff::try_from(diff)?;

        Ok::<_, Error>(diff.into())
    }

    /// As `get_diff`, for a commit against its first parent.
    fn get_commit_diff(&self, rid: identity::RepoId, sha: git::Oid) -> Result<Diff, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?.backend;
        let diff = tree_diff(&repo, None, sha, 0, false)?;
        let diff = surf::diff::Diff::try_from(diff)?;

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

        // Hide `base` from the walk rather than stopping at the first commit
        // that equals it. A merge commit reaches `base` through one of its
        // parents, so truncating there drops every commit the other parent
        // contributes and leaves the revision looking like a single commit.
        let mut walk = repo.backend.revwalk()?;
        walk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
        walk.push(git::raw::Oid::from_str(&head)?)?;
        walk.hide(git::raw::Oid::from_str(&base)?)?;

        let aliases = profile.aliases();
        let signers = Signers::new(&repo, rid, &profile, &aliases);

        let surf_repo = surf::Repository::open(repo.path())?;
        let commits = walk
            .filter_map(|oid| oid.ok())
            .filter_map(|oid| surf_repo.commit(git::Oid::from(oid)).ok())
            .map(|commit| {
                let signature = signers.verify(commit.id);
                repo::Commit::from(commit).with_signature(signature)
            })
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

        let aliases = profile.aliases();
        let signers = Signers::new(&storage_repo, rid, &profile, &aliases);

        let repo = surf::Repository::open(storage_repo.path())?;
        let commits = repo.history(oid)?;
        let cursor = skip.unwrap_or(0);
        // Signature checking runs after `skip`/`take` so that paging through a
        // long history only verifies the page being returned.
        let sign = |commit: surf::Commit| {
            let signature = signers.verify(commit.id);
            repo::Commit::from(commit).with_signature(signature)
        };

        match take {
            None => {
                let content: Vec<repo::Commit> = commits.filter_map(|c| c.ok()).map(sign).collect();

                Ok(crate::cobs::PaginatedQuery {
                    cursor: 0,
                    more: false,
                    content,
                })
            }
            Some(take) => {
                let content: Vec<repo::Commit> = commits
                    .filter_map(|c| c.ok())
                    .skip(cursor)
                    .take(take + 1)
                    .map(sign)
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

        let aliases = profile.aliases();
        let signers = Signers::new(&storage_repo, rid, &profile, &aliases);

        let repo = surf::Repository::open(storage_repo.path())?;
        let commit = repo.commit(oid)?;
        let signature = signers.verify(commit.id);

        Ok(repo::Commit::from(commit).with_signature(signature))
    }

    fn unseed(&self, rid: identity::RepoId) -> Result<(), Error> {
        let profile = self.profile();
        let mut node = radicle::Node::new(profile.home().socket_from_env());

        profile.unseed(rid, &mut node)?;

        Ok(())
    }

    /// Remove the repository's remotes from storage, mirroring `rad clean`.
    ///
    /// If the local node has never written signed refs for this repository,
    /// storage drops it entirely. Otherwise only the remotes that are neither
    /// the local node's nor a delegate's are removed, and the repository stays
    /// on disk.
    fn clean(&self, rid: identity::RepoId) -> Result<(), Error> {
        let profile = self.profile();

        profile.storage.clean(rid)?;

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

#[cfg(test)]
mod test {
    use super::*;

    /// A real commit object produced by `git commit -S` with `gpg.format=ssh`
    /// and a Radicle key as `user.signingKey`. Using git's own output rather
    /// than a signature this crate produced is the point: it pins down the
    /// payload git actually signs, not our reading of the format.
    const SIGNED_COMMIT: &[u8] = b"\
tree 96c45f4710a3a9e9268f13f56d6a7308b463a023\n\
parent e35baf750ffce50ac03f379e7cfca88dd3ccaa2d\n\
author R\xc5\xabdolfs O\xc5\xa1i\xc5\x86\xc5\xa1 <rudolfs@osins.org> 1788360297 +0200\n\
committer R\xc5\xabdolfs O\xc5\xa1i\xc5\x86\xc5\xa1 <rudolfs@osins.org> 1788452563 +0200\n\
gpgsig -----BEGIN SSH SIGNATURE-----\n\
\x20U1NIU0lHAAAAAQAAADMAAAALc3NoLWVkMjU1MTkAAAAg+56aXUXHOO/BsPu9Fl/ChqjK7E\n\
\x20vLQv5YsHBI2JWgl7kAAAADZ2l0AAAAAAAAAAZzaGE1MTIAAABTAAAAC3NzaC1lZDI1NTE5\n\
\x20AAAAQAqwcPwx+FD1Fro6tplXTtTuFO2MMFkk+atB9h6up2xmYrbTXZNzMIfnpTdU3lseuL\n\
\x20hGCjr3Id4E588iFaWivA4=\n\
\x20-----END SSH SIGNATURE-----\n\
\n\
Fix patch diff for merge-commit revisions\n\
\n\
`list_commits` walked back from the revision head and stopped at the\n\
first commit equal to `base`. A merge commit reaches `base` through one\n\
of its parents, so the walk truncated there and dropped everything the\n\
other parent contributed: a revision whose head merges the target branch\n\
in looked like a single-commit revision.\n\
";

    /// Write a raw commit object into a scratch repository. The tree and parent
    /// it names are not present, which the object database does not require.
    fn repo_with(object: &[u8]) -> (tempfile::TempDir, git::raw::Repository, git::Oid) {
        let dir = tempfile::tempdir().unwrap();
        let repo = git::raw::Repository::init(dir.path()).unwrap();
        let oid = repo
            .odb()
            .unwrap()
            .write(git::raw::ObjectType::Commit, object)
            .unwrap();

        (dir, repo, oid.into())
    }

    #[test]
    fn verifies_a_git_ssh_signature() {
        let (_dir, repo, oid) = repo_with(SIGNED_COMMIT);
        let (status, signer) = verify_signature(&repo, oid).unwrap();

        assert_eq!(status, repo::SignatureStatus::Verified);
        // This fixture was signed with a Radicle key, so the signing key is
        // also the signer's node ID.
        assert_eq!(
            signer.unwrap().to_string(),
            "z6MkwPUeUS2fJMfc2HZN1RQTQcTTuhw4HhPySB8JeUg2mVvx"
        );
    }

    #[test]
    fn rejects_a_tampered_payload() {
        // The signature covers the whole commit object bar the `gpgsig` header,
        // so editing the message must invalidate it.
        let tampered = String::from_utf8_lossy(SIGNED_COMMIT)
            .replace("merge-commit", "merge-commlt")
            .into_bytes();
        let (_dir, repo, oid) = repo_with(&tampered);
        let (status, signer) = verify_signature(&repo, oid).unwrap();

        assert_eq!(status, repo::SignatureStatus::Invalid);
        assert!(signer.is_some());
    }

    #[test]
    fn reports_an_unsigned_commit() {
        let unsigned = b"\
tree 96c45f4710a3a9e9268f13f56d6a7308b463a023\n\
author Alice <alice@example.com> 1788360297 +0200\n\
committer Alice <alice@example.com> 1788360297 +0200\n\
\n\
Initial commit\n";
        let (_dir, repo, oid) = repo_with(unsigned);

        assert!(verify_signature(&repo, oid).is_none());
    }

    #[test]
    fn does_not_accept_another_namespace() {
        // Radicle signs collaborative objects with the same key and container
        // but under the `radicle` namespace, over a commit id rather than a
        // commit payload. Such a signature says nothing about this commit.
        let mut sig = ssh_key::SshSig::from_pem(
            &String::from_utf8(SIGNED_COMMIT.to_vec())
                .unwrap()
                .lines()
                .skip_while(|l| !l.starts_with("gpgsig "))
                .take_while(|l| !l.is_empty())
                .map(|l| l.trim_start_matches("gpgsig ").trim_start())
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .unwrap();
        sig = ssh_key::SshSig::new(
            sig.public_key().clone(),
            "radicle",
            sig.hash_alg(),
            sig.signature().clone(),
        )
        .unwrap();
        let pem = sig.to_pem(ssh_key::LineEnding::LF).unwrap();

        let mut object = b"\
tree 96c45f4710a3a9e9268f13f56d6a7308b463a023\n\
author Alice <alice@example.com> 1788360297 +0200\n\
committer Alice <alice@example.com> 1788360297 +0200\n\
gpgsig "
            .to_vec();
        object.extend(pem.trim_end().replace('\n', "\n ").as_bytes());
        object.extend(b"\n\nInitial commit\n");

        let (_dir, repo, oid) = repo_with(&object);
        let (status, signer) = verify_signature(&repo, oid).unwrap();

        assert_eq!(status, repo::SignatureStatus::Unsupported);
        assert!(signer.is_none());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod identity_test {
    use radicle::cob;
    use radicle::crypto::{Seed, Signer as _, SigningKey};
    use radicle::identity::Visibility;

    use super::*;

    struct Fixture {
        _tmp: tempfile::TempDir,
        profile: radicle::Profile,
        rid: identity::RepoId,
    }

    /// A storage repository whose identity is delegated to a single key, so
    /// that every revision this key proposes reaches quorum on its own and is
    /// adopted immediately.
    fn fixture() -> (Fixture, SigningKey) {
        let tmp = tempfile::tempdir().unwrap();
        let profile = crate::test::profile(tmp.path(), [0xff; 32]);
        let signer = SigningKey::from_seed(Seed::new([0xff; 32]));

        let (working, _) = radicle::test::fixtures::repository(tmp.path().join("working"));
        let (rid, _, _) = radicle::rad::init(
            &working,
            "acme".try_into().unwrap(),
            "",
            radicle::git::fmt::refname!("master"),
            Visibility::default(),
            &signer,
            &profile.storage,
        )
        .unwrap();

        (
            Fixture {
                _tmp: tmp,
                profile,
                rid,
            },
            signer,
        )
    }

    fn historical(fixture: &Fixture) -> BTreeSet<identity::Did> {
        let repo = fixture.profile.storage.repository(fixture.rid).unwrap();
        let aliases = fixture.profile.aliases();

        Signers::new(&repo, fixture.rid, &fixture.profile, &aliases).historical
    }

    #[test]
    fn remembers_a_rescinded_delegate() {
        let (fixture, signer) = fixture();
        let repo = fixture.profile.storage.repository(fixture.rid).unwrap();
        let bob = identity::Did::from(*SigningKey::from_seed(Seed::new([0x01; 32])).public_key());

        let mut identity = cob::identity::Identity::load_mut(&repo, &signer).unwrap();
        let mut doc = identity.doc().clone().edit();
        doc.delegate(bob);
        identity
            .update(
                cob::Title::new("Add Bob").unwrap(),
                "",
                &doc.clone().verified().unwrap(),
            )
            .unwrap();

        let mut doc = identity.doc().clone().edit();
        doc.rescind(&bob).unwrap();
        identity
            .update(
                cob::Title::new("Remove Bob").unwrap(),
                "",
                &doc.verified().unwrap(),
            )
            .unwrap();

        let repo = fixture.profile.storage.repository(fixture.rid).unwrap();
        let DocAt { doc, .. } = repo.identity_doc().unwrap();
        assert!(!doc.delegates().contains(&bob), "Bob is no longer current");
        assert!(historical(&fixture).contains(&bob), "Bob is remembered");
    }

    #[test]
    fn ignores_a_proposal_that_never_reached_quorum() {
        // The security property: a revision that was merely *proposed* must not
        // confer delegate history, or a single delegate could name anyone.
        let (fixture, signer) = fixture();
        let repo = fixture.profile.storage.repository(fixture.rid).unwrap();
        let bob = identity::Did::from(*SigningKey::from_seed(Seed::new([0x01; 32])).public_key());
        let eve = identity::Did::from(*SigningKey::from_seed(Seed::new([0x02; 32])).public_key());

        // Raise the threshold to two so the next proposal cannot self-adopt.
        let mut identity = cob::identity::Identity::load_mut(&repo, &signer).unwrap();
        let mut doc = identity.doc().clone().edit();
        doc.delegate(bob);
        doc.threshold = 2;
        identity
            .update(
                cob::Title::new("Add Bob").unwrap(),
                "",
                &doc.verified().unwrap(),
            )
            .unwrap();

        let mut doc = identity.doc().clone().edit();
        doc.delegate(eve);
        let proposal = identity
            .update(
                cob::Title::new("Add Eve").unwrap(),
                "",
                &doc.verified().unwrap(),
            )
            .unwrap();

        assert!(
            identity.revision(&proposal).unwrap().is_active(),
            "the proposal is unaccepted"
        );
        assert!(!historical(&fixture).contains(&eve), "Eve does not count");
        assert!(historical(&fixture).contains(&bob), "Bob does count");
    }
}
