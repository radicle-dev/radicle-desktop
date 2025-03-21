use std::collections::BTreeSet;
use std::str::FromStr;
use std::sync::Arc;

use localtime::LocalTime;
use radicle::cob;
use radicle::crypto::ssh::Passphrase;
use radicle::identity::project::ProjectName;
use radicle::identity::{doc, DocAt, Visibility};
use radicle::issue::cache::Issues;
use radicle::issue::IssueId;
use radicle::node::{Alias, Handle};
use radicle::patch::cache::Patches;
use radicle::patch::{Patch, PatchId};
use radicle::prelude::Project;
use radicle::profile::env;
use radicle::rad::InitError;
use radicle::storage::git::Repository;
use radicle::storage::refs::branch_of;
use radicle::storage::{
    self, ReadRepository, ReadStorage, RepositoryInfo, SignRepository, WriteRepository,
};
use radicle::{git, Node};
use radicle::{identity, Profile};
use radicle_surf as surf;
use serde::de::DeserializeOwned;

use crate::domain::identity::traits::IdentityService;
use crate::domain::repo::models::cobs::patch::ListPatchesError;
use crate::domain::repo::models::cobs::CobOptions;
use crate::domain::repo::models::repo::repo_info;
use crate::domain::repo::models::syntax::ToPretty;
use crate::domain::repo::models::{cobs, repo, syntax};
use crate::domain::repo::traits::cobs::RepoActivity;
use crate::domain::repo::traits::issue::RepoIssues;
use crate::domain::repo::traits::patch::{RepoPatches, RepoPatchesLister};
use crate::domain::repo::traits::repo::RepoStorage;
use crate::domain::repo::traits::thread::RepoThreads;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Radicle {
    profile: Arc<Profile>,
}

impl Radicle {
    pub fn new(profile: Profile) -> Self {
        Self {
            profile: Arc::new(profile),
        }
    }

    pub fn init(alias: String, passphrase: Passphrase) -> Result<Self, Error> {
        let home = radicle::profile::home()?;
        let alias = Alias::from_str(&alias)?;

        if passphrase.is_empty() {
            return Err(Error::Crypto(
                radicle::crypto::ssh::keystore::Error::PassphraseMissing,
            ));
        }
        let profile = radicle::Profile::init(home, alias, Some(passphrase.clone()), env::seed())?;
        match radicle::crypto::ssh::agent::Agent::connect() {
            Ok(mut agent) => Self::register(&mut agent, &profile, passphrase.clone())?,
            Err(e) if e.is_not_running() => return Err(Error::AgentNotRunning),
            Err(e) => Err(e)?,
        }

        Ok(Self {
            profile: Arc::new(profile),
        })
    }

    pub fn register(
        agent: &mut radicle::crypto::ssh::agent::Agent,
        profile: &radicle::Profile,
        passphrase: radicle::crypto::ssh::Passphrase,
    ) -> Result<(), Error> {
        let secret = profile
            .keystore
            .secret_key(Some(passphrase))
            .map_err(|e| {
                if e.is_crypto_err() {
                    Error::Crypto(radicle::crypto::ssh::keystore::Error::Ssh(
                        ssh_key::Error::Crypto,
                    ))
                } else {
                    e.into()
                }
            })?
            .ok_or(Error::Crypto(radicle::crypto::ssh::keystore::Error::Ssh(
                ssh_key::Error::Crypto,
            )))?;

        agent.register(&secret)?;

        Ok(())
    }

    pub fn profile(&self) -> Arc<Profile> {
        self.profile.clone()
    }
}

impl IdentityService for Radicle {
    fn authenticate(&self, passphrase: Passphrase) -> Result<(), Error> {
        let profile = (*self.profile).clone();
        if !profile.keystore.is_encrypted()? {
            return Ok(());
        }
        match radicle::crypto::ssh::agent::Agent::connect() {
            Ok(mut agent) => {
                if agent.request_identities()?.contains(&profile.public_key) {
                    return Ok(());
                }

                profile.keystore.secret_key(Some(passphrase.clone()))?;
                Self::register(&mut agent, &profile, passphrase)
            }
            Err(e) if e.is_not_running() => Err(Error::AgentNotRunning)?,
            Err(e) => Err(e)?,
        }
    }
}

impl RepoStorage for Radicle {
    fn create_repo(&self, name: String, description: String) -> Result<(), Error> {
        let profile = (*self.profile).clone();
        let storage = &profile.storage;
        let signer = profile.signer()?;
        let config = radicle::git::raw::Config::open_default()?;
        // SAFETY: "master" is always a valid RefString
        let default_branch = git::RefString::try_from(
            config
                .get_string("init.defaultBranch")
                .unwrap_or("master".to_owned()),
        )
        .unwrap();

        let name = ProjectName::from_str(&name)?;
        if description.len() > doc::MAX_STRING_LENGTH {
            return Err(Error::ProjectError(
                radicle::identity::project::ProjectError::Description(
                    "Cannot exceed 255 characters.",
                ),
            ));
        }

        let visibility = Visibility::Private {
            allow: BTreeSet::default(),
        };

        let proj = Project::new(name, description, default_branch.clone()).map_err(|errs| {
            InitError::ProjectPayload(
                errs.into_iter()
                    .map(|err| err.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        })?;
        let doc = radicle::identity::Doc::initial(proj, profile.public_key.into(), visibility);
        let (project, identity) = Repository::init(&doc, &storage, &signer)?;

        let tree_id = {
            let mut index = project.backend.index()?;

            index.write_tree()
        }?;
        let sig = project.backend.signature()?;
        let tree = project.backend.find_tree(tree_id)?;

        project.set_remote_identity_root_to(signer.public_key(), identity)?;
        project.set_identity_head_to(identity)?;

        let base =
            project
                .backend
                .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

        let ns_head = branch_of(&profile.public_key, &default_branch);
        project
            .backend
            .reference(ns_head.as_str(), base, false, "Created namespace ref")?;

        project.set_head()?;
        project.sign_refs(&signer)?;

        Ok(())
    }

    fn list_repos(
        &self,
        show: crate::domain::repo::models::repo::Show,
    ) -> Result<Vec<crate::domain::repo::models::repo::RepoInfo>, Error> {
        let profile = (*self.profile).clone();
        let storage = &profile.storage;
        let policies = profile.policies()?;
        let repos = storage.repositories()?;
        let mut entries = Vec::new();

        for RepositoryInfo { rid, doc, refs, .. } in repos {
            if refs.is_none() && show == repo::Show::Contributor {
                continue;
            }

            if !policies.is_seeding(&rid)? && show == repo::Show::Seeded {
                continue;
            }

            if !doc.is_private() && show == repo::Show::Private {
                continue;
            }

            if !doc.delegates().contains(&profile.public_key.into()) && show == repo::Show::Delegate
            {
                continue;
            }

            let repo = profile.storage.repository(rid)?;
            let repo_info = repo_info(&profile, &repo, &doc)?;

            entries.push(repo_info)
        }

        entries.sort_by(|a, b| b.last_commit_timestamp.cmp(&a.last_commit_timestamp));

        Ok::<_, Error>(entries)
    }

    fn repo_count(&self) -> Result<crate::domain::repo::models::repo::RepoCount, Error> {
        let profile = (*self.profile).clone();
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

        Ok::<_, Error>(repo::RepoCount {
            total,
            contributor,
            seeding,
            private,
            delegate,
        })
    }

    fn repo_by_id(
        &self,
        rid: identity::RepoId,
    ) -> Result<crate::domain::repo::models::repo::RepoInfo, Error> {
        let profile = (*self.profile).clone();
        let repo = profile.storage.repository(rid)?;
        let DocAt { doc, .. } = repo.identity_doc()?;

        let repo_info = repo_info(&profile, &repo, &doc)?;

        Ok::<_, Error>(repo_info)
    }

    fn diff_stats(
        &self,
        rid: identity::RepoId,
        base: git::Oid,
        head: git::Oid,
    ) -> Result<cobs::Stats, Error> {
        let profile = (*self.profile).clone();
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

    fn get_diff(
        &self,
        rid: identity::RepoId,
        options: crate::domain::repo::models::diff::Options,
    ) -> Result<crate::domain::repo::models::diff::Diff, Error> {
        let profile = (*self.profile).clone();
        let unified = options.unified.unwrap_or(5);
        let highlight = options.highlight.unwrap_or(true);
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
            let mut hi = syntax::Highlighter::new();

            return Ok::<_, Error>(diff.pretty(&mut hi, &(), &repo));
        }

        Ok::<_, Error>(diff.into())
    }

    fn list_commits(
        &self,
        rid: identity::RepoId,
        base: git::Oid,
        head: git::Oid,
    ) -> Result<Vec<crate::domain::repo::models::repo::Commit>, Error> {
        let profile = (*self.profile).clone();
        let repo = profile.storage.repository(rid)?;

        let repo = surf::Repository::open(repo.path())?;
        let history = repo.history(head)?;

        let commits = history
            .take_while(|c| {
                if let Ok(c) = c {
                    c.id.to_string() != base.to_string()
                } else {
                    false
                }
            })
            .filter_map(|c| c.map(Into::into).ok())
            .collect();

        Ok(commits)
    }
}

impl RepoThreads for Radicle {
    fn get_embed(
        &self,
        rid: identity::RepoId,
        name: Option<String>,
        oid: git::Oid,
    ) -> Result<cobs::EmbedWithMimeType, Error> {
        let profile = (*self.profile).clone();
        let repo = profile.storage.repository(rid)?;
        let blob = repo.blob(oid)?;
        let content = blob.content();
        let mime_type = match infer::get(content).map(|i| i.mime_type().to_string()) {
            Some(mime_type) => Some(mime_type),
            None if name.is_some() => {
                let filename = name.unwrap();
                mime_infer::from_path(&filename)
                    .first()
                    .map(|m| m.as_ref().to_string())
            }
            _ => None,
        };

        Ok::<_, Error>(cobs::EmbedWithMimeType {
            content: content.to_vec(),
            mime_type,
        })
    }

    fn save_embed_to_disk(
        &self,
        rid: identity::RepoId,
        oid: git::Oid,
        path: std::path::PathBuf,
    ) -> Result<(), Error> {
        let profile = (*self.profile).clone();
        let repo = profile.storage.repository(rid)?;
        let blob = repo.blob(oid)?;
        std::fs::write(path, blob.content())?;

        Ok::<_, Error>(())
    }

    fn save_embed_by_path(
        &self,
        rid: identity::RepoId,
        path: std::path::PathBuf,
    ) -> Result<git::Oid, Error> {
        let profile = (*self.profile).clone();
        let repo = profile.storage.repository(rid)?;
        let bytes = std::fs::read(path.clone())?;
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("embed");
        let embed = radicle::cob::Embed::<git::Oid>::store(name, &bytes, &repo.backend)?;

        Ok(embed.oid())
    }

    fn save_embed_by_bytes(
        &self,
        rid: identity::RepoId,
        name: String,
        bytes: Vec<u8>,
    ) -> Result<git::Oid, Error> {
        let profile = (*self.profile).clone();
        let repo = profile.storage.repository(rid)?;
        let embed = radicle::cob::Embed::<git::Oid>::store(&name, &bytes, &repo.backend)?;

        Ok(embed.oid())
    }

    fn create_issue_comment(
        &self,
        rid: identity::RepoId,
        new: cobs::thread::NewIssueComment,
        opts: cobs::CobOptions,
    ) -> Result<cobs::thread::Comment<cobs::Never>, Error> {
        let profile = (*self.profile).clone();
        let aliases = &profile.aliases();
        let mut node = Node::new(profile.socket());
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;
        let mut issues = profile.issues_mut(&repo)?;
        let mut issue = issues.get_mut(&new.id.into())?;
        let id = new.reply_to.unwrap_or_else(|| {
            let (root_id, _) = issue.root();
            *root_id
        });
        let n = new.clone();
        let oid = issue.comment(
            n.body,
            id,
            n.embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
            &signer,
        )?;

        if opts.announce() {
            if let Err(e) = node.announce_refs(rid) {
                log::error!("Not able to announce changes: {}", e)
            }
        }

        Ok(cobs::thread::Comment::<cobs::Never>::new(
            oid,
            cob::thread::Comment::new(
                *signer.public_key(),
                new.body,
                id.into(),
                None,
                new.embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                LocalTime::now().into(),
            ),
            aliases,
        ))
    }

    fn create_patch_comment(
        &self,
        rid: identity::RepoId,
        new: cobs::thread::NewPatchComment,
        opts: cobs::CobOptions,
    ) -> Result<cobs::thread::Comment<cobs::thread::CodeLocation>, Error> {
        let profile = (*self.profile).clone();
        let aliases = &profile.aliases();
        let mut node = Node::new(profile.socket());
        let signer = profile.signer()?;
        let repo = profile.storage.repository(rid)?;
        let mut patches = profile.patches_mut(&repo)?;
        let mut patch = patches.get_mut(&new.id.into())?;
        let n = new.clone();
        let oid = patch.comment(
            new.revision.into(),
            n.body,
            n.reply_to,
            n.location.map(|l| l.into()),
            n.embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
            &signer,
        )?;

        if opts.announce() {
            if let Err(e) = node.announce_refs(rid) {
                log::error!("Not able to announce changes: {}", e)
            }
        }

        Ok(cobs::thread::Comment::<cobs::thread::CodeLocation>::new(
            oid,
            cob::thread::Comment::new(
                *signer.public_key(),
                new.body,
                new.reply_to,
                new.location.map(|l| l.into()),
                new.embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                LocalTime::now().into(),
            ),
            aliases,
        ))
    }
}

impl RepoIssues for Radicle {
    fn list_issues(
        &self,
        rid: identity::RepoId,
        status: Option<cobs::query::IssueStatus>,
    ) -> Result<Vec<cobs::issue::Issue>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let status = status.unwrap_or_default();
        let issues = profile.issues(&repo)?;
        let mut issues: Vec<_> = issues
            .list()?
            .filter_map(|r| {
                let (id, issue) = r.ok()?;
                (status.matches(issue.state())).then_some((id, issue))
            })
            .collect::<Vec<_>>();

        issues.sort_by(|(_, a), (_, b)| b.timestamp().cmp(&a.timestamp()));
        let aliases = &profile.aliases();
        let issues = issues
            .into_iter()
            .map(|(id, issue)| cobs::issue::Issue::new(&id, &issue, aliases))
            .collect::<Vec<_>>();

        Ok::<_, Error>(issues)
    }

    fn issue_by_id(
        &self,
        rid: identity::RepoId,
        id: IssueId,
    ) -> Result<Option<cobs::issue::Issue>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let issues = profile.issues(&repo)?;
        let issue = issues.get(&id)?;

        let aliases = &profile.aliases();
        let issue = issue.map(|issue| cobs::issue::Issue::new(&id, &issue, aliases));

        Ok::<_, Error>(issue)
    }

    fn comment_threads_by_issue_id(
        &self,
        rid: identity::RepoId,
        id: IssueId,
    ) -> Result<Option<Vec<cobs::thread::Thread>>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let issues = profile.issues(&repo)?;
        let issue = issues.get(&id)?;

        let aliases = &profile.aliases();
        let comments = issue.map(|issue| {
            issue
                .replies()
                // Filter out replies that aren't top level replies
                .filter(|c| {
                    let Some(oid) = c.1.reply_to() else {
                        return false;
                    };

                    oid == *id
                })
                .map(|(oid, c)| {
                    let root = cobs::thread::Comment::<cobs::Never>::new(*oid, c.clone(), aliases);
                    let replies = issue
                        .replies_to(oid)
                        .map(|(oid, c)| {
                            cobs::thread::Comment::<cobs::Never>::new(*oid, c.clone(), aliases)
                        })
                        .collect::<Vec<_>>();

                    cobs::thread::Thread { root, replies }
                })
                .collect::<Vec<_>>()
        });

        Ok::<_, Error>(comments)
    }

    fn create_issue(
        &self,
        rid: identity::RepoId,
        new: cobs::issue::NewIssue,
        opts: cobs::CobOptions,
    ) -> Result<cobs::issue::Issue, Error> {
        let profile = self.profile();
        let mut node = Node::new(profile.socket());
        let repo = profile.storage.repository(rid)?;
        let signer = profile.signer()?;
        let aliases = profile.aliases();
        let mut issues = profile.issues_mut(&repo)?;
        let issue = issues.create(
            new.title,
            new.description,
            &new.labels,
            &new.assignees,
            new.embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
            &signer,
        )?;

        if opts.announce() {
            if let Err(e) = node.announce_refs(rid) {
                log::error!("Not able to announce changes: {}", e)
            }
        }

        Ok::<_, Error>(cobs::issue::Issue::new(issue.id(), &issue, &aliases))
    }

    fn edit_issue(
        &self,
        rid: identity::RepoId,
        cob_id: IssueId,
        action: cobs::issue::Action,
        opts: cobs::CobOptions,
    ) -> Result<cobs::issue::Issue, Error> {
        let profile = self.profile();
        let mut node = Node::new(profile.socket());
        let repo = profile.storage.repository(rid)?;
        let signer = profile.signer()?;
        let aliases = profile.aliases();
        let mut issues = profile.issues_mut(&repo)?;
        let mut issue = issues.get_mut(&cob_id)?;

        match action {
            cobs::issue::Action::Lifecycle { state } => {
                issue.lifecycle(state.into(), &signer)?;
            }
            cobs::issue::Action::Assign { assignees } => {
                issue.assign(
                    assignees.iter().map(|a| *a.did()).collect::<BTreeSet<_>>(),
                    &signer,
                )?;
            }
            cobs::issue::Action::Label { labels } => {
                issue.label(labels, &signer)?;
            }
            cobs::issue::Action::CommentReact {
                id,
                reaction,
                active,
            } => {
                issue.react(id, reaction, active, &signer)?;
            }
            cobs::issue::Action::CommentRedact { id } => {
                issue.redact_comment(id, &signer)?;
            }
            cobs::issue::Action::Comment {
                body,
                reply_to,
                embeds,
            } => {
                issue.comment(
                    body,
                    reply_to.unwrap_or(*cob_id),
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::issue::Action::CommentEdit { id, body, embeds } => {
                issue.edit_comment(
                    id,
                    body,
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::issue::Action::Edit { title } => {
                issue.edit(title, &signer)?;
            }
        }

        if opts.announce() {
            if let Err(e) = node.announce_refs(rid) {
                log::error!("Not able to announce changes: {}", e)
            }
        }

        Ok::<_, Error>(cobs::issue::Issue::new(issue.id(), &issue, &aliases))
    }
}

impl RepoPatchesLister for Radicle {
    fn list(
        &self,
        _rid: identity::RepoId,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, ListPatchesError> {
        Err::<std::iter::Empty<_>, ListPatchesError>(ListPatchesError::Unimplemented)
    }

    fn list_by_status(
        &self,
        _rid: identity::RepoId,
        _status: radicle::patch::Status,
    ) -> Result<impl Iterator<Item = (PatchId, Patch)>, ListPatchesError> {
        Err::<std::iter::Empty<_>, ListPatchesError>(ListPatchesError::Unimplemented)
    }
}

impl RepoPatches for Radicle {
    fn get_patch_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
    ) -> Result<Option<cobs::patch::Patch>, Error> {
        let profile = (*self.profile).clone();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo)?;
        let patch = patches.get(&id)?;
        let aliases = &profile.aliases();

        Ok(patch.map(|patch| cobs::patch::Patch::new(&id, &patch, aliases)))
    }

    fn revisions_by_patch(
        &self,
        rid: identity::RepoId,
        id: PatchId,
    ) -> Result<Option<Vec<cobs::patch::Revision>>, Error> {
        let profile = (*self.profile).clone();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo)?;
        let revisions = patches.get(&id)?.map(|patch| {
            let aliases = &profile.aliases();

            patch
                .revisions()
                .map(|(_, r)| cobs::patch::Revision::new(r.clone(), aliases))
                .collect::<Vec<_>>()
        });

        Ok::<_, Error>(revisions)
    }

    fn revision_by_patch_and_id(
        &self,
        _rid: identity::RepoId,
        _id: PatchId,
        _revision_id: radicle::patch::RevisionId,
    ) -> Result<Option<cobs::patch::Revision>, Error> {
        todo!()
    }

    fn revision_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: radicle::patch::RevisionId,
    ) -> Result<Option<cobs::patch::Revision>, Error> {
        let profile = (*self.profile).clone();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo)?;
        let revision = patches.get(&id)?.and_then(|patch| {
            let aliases = &profile.aliases();

            patch
                .revision(&revision_id)
                .map(|r| cobs::patch::Revision::new(r.clone(), aliases))
        });

        Ok::<_, Error>(revision)
    }

    fn review_by_id(
        &self,
        rid: identity::RepoId,
        id: PatchId,
        revision_id: radicle::patch::RevisionId,
        review_id: radicle::patch::ReviewId,
    ) -> Result<Option<cobs::patch::Review>, Error> {
        let profile = (*self.profile).clone();
        let repo = profile.storage.repository(rid)?;
        let patches = profile.patches(&repo)?;
        let review = patches.get(&id)?.and_then(|patch| {
            let aliases = &profile.aliases();

            patch
                .reviews_of(revision_id)
                .find(|(id, _)| *id == &review_id)
                .map(|(_, review)| cobs::patch::Review::new(review.clone(), aliases))
        });

        Ok::<_, Error>(review)
    }

    fn edit_patch(
        &self,
        rid: identity::RepoId,
        cob_id: PatchId,
        action: cobs::patch::Action,
        opts: CobOptions,
    ) -> Result<cobs::patch::Patch, Error> {
        let profile = (*self.profile).clone();
        let mut node = Node::new(profile.socket());
        let repo = profile.storage.repository(rid)?;
        let signer = profile.signer()?;
        let aliases = profile.aliases();
        let mut patches = profile.patches_mut(&repo)?;
        let mut patch = patches.get_mut(&cob_id)?;

        match action {
            cobs::patch::Action::RevisionEdit {
                revision,
                description,
                embeds,
            } => {
                patch.edit_revision(
                    revision,
                    description,
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::RevisionCommentRedact { revision, comment } => {
                patch.comment_redact(revision, comment, &signer)?;
            }
            cobs::patch::Action::ReviewCommentRedact { review, comment } => {
                patch.redact_review_comment(review, comment, &signer)?;
            }
            cobs::patch::Action::ReviewCommentReact {
                review,
                comment,
                reaction,
                active,
            } => {
                patch.react_review_comment(review, comment, reaction, active, &signer)?;
            }
            cobs::patch::Action::ReviewCommentResolve { review, comment } => {
                patch.resolve_review_comment(review, comment, &signer)?;
            }
            cobs::patch::Action::ReviewCommentUnresolve { review, comment } => {
                patch.unresolve_review_comment(review, comment, &signer)?;
            }
            cobs::patch::Action::Edit { title, target } => {
                patch.edit(title, target, &signer)?;
            }
            cobs::patch::Action::ReviewEdit {
                review,
                summary,
                verdict,
                labels,
            } => {
                patch.review_edit(review, verdict.map(|v| v.into()), summary, labels, &signer)?;
            }
            cobs::patch::Action::Review {
                revision,
                summary,
                verdict,
                labels,
            } => {
                patch.review(
                    revision,
                    verdict.map(|v| v.into()),
                    summary,
                    labels,
                    &signer,
                )?;
            }
            cobs::patch::Action::ReviewRedact { review } => {
                patch.redact_review(review, &signer)?;
            }
            cobs::patch::Action::ReviewComment {
                review,
                body,
                location,
                reply_to,
                embeds,
            } => {
                patch.review_comment(
                    review,
                    body,
                    location.map(|l| l.into()),
                    reply_to,
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::ReviewCommentEdit {
                review,
                comment,
                body,
                embeds,
            } => {
                patch.edit_review_comment(
                    review,
                    comment,
                    body,
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::Lifecycle { state } => {
                patch.lifecycle(state, &signer)?;
            }
            cobs::patch::Action::Assign { assignees } => {
                patch.assign(
                    assignees.iter().map(|a| *a.did()).collect::<BTreeSet<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::Label { labels } => {
                patch.label(labels, &signer)?;
            }
            cobs::patch::Action::RevisionReact {
                revision,
                reaction,
                location,
                active,
            } => {
                patch.react(
                    revision,
                    reaction,
                    location.map(|l| l.into()),
                    active,
                    &signer,
                )?;
            }
            cobs::patch::Action::RevisionComment {
                revision,
                location,
                body,
                reply_to,
                embeds,
            } => {
                patch.comment(
                    revision,
                    body,
                    reply_to,
                    location.map(|l| l.into()),
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::RevisionCommentEdit {
                revision,
                comment,
                body,
                embeds,
            } => {
                patch.comment_edit(
                    revision,
                    comment,
                    body,
                    embeds.into_iter().map(Into::into).collect::<Vec<_>>(),
                    &signer,
                )?;
            }
            cobs::patch::Action::RevisionCommentReact {
                revision,
                comment,
                reaction,
                active,
            } => {
                patch.comment_react(revision, comment, reaction, active, &signer)?;
            }
            cobs::patch::Action::RevisionRedact { revision } => {
                patch.redact(revision, &signer)?;
            }
            cobs::patch::Action::Merge { .. } => {
                unimplemented!("We don't support merging of patches through the desktop")
            }
            cobs::patch::Action::Revision { .. } => {
                unimplemented!("We don't support creating new revisions through the desktop")
            }
        }

        if opts.announce() {
            if let Err(e) = node.announce_refs(rid) {
                log::error!("Not able to announce changes: {}", e)
            }
        }

        Ok::<_, Error>(cobs::patch::Patch::new(patch.id(), &patch, &aliases))
    }
}

impl RepoActivity for Radicle {
    fn activity_by_id<A: DeserializeOwned, B: cobs::FromRadicleAction<A>>(
        &self,
        rid: identity::RepoId,
        type_name: &cob::TypeName,
        id: git::Oid,
    ) -> Result<Vec<cobs::Operation<B>>, Error> {
        let profile = (*self.profile).clone();
        let aliases = profile.aliases();
        let repo = profile.storage.repository(rid)?;
        let iter = cob::store::ops(&id.into(), type_name, &repo)?;
        let ops = iter
            .into_iter()
            .map(|op| {
                let actions = op
                    .actions
                    .iter()
                    .filter_map(|a| {
                        if let Ok(r) = serde_json::from_slice::<A>(a) {
                            let x = B::from_radicle_action(r, &aliases);
                            Some(x)
                        } else {
                            log::error!("Not able to deserialize the action");

                            None
                        }
                    })
                    .collect::<Vec<_>>();

                cobs::Operation {
                    id: op.id,
                    actions,
                    author: cobs::Author::new(&op.author.into(), &aliases),
                    timestamp: op.timestamp,
                }
            })
            .collect::<Vec<_>>();

        Ok::<_, Error>(ops)
    }
}
