use std::collections::BTreeSet;

use radicle::issue::cache::Issues as _;
use radicle::node::Handle;
use radicle::storage::ReadStorage;
use radicle::{git, identity, Node};

use crate::cobs;
use crate::error::Error;
use crate::traits::Profile;

pub trait Issues: Profile {
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
        id: git::Oid,
    ) -> Result<Option<cobs::issue::Issue>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let issues = profile.issues(&repo)?;
        let issue = issues.get(&id.into())?;

        let aliases = &profile.aliases();
        let issue = issue.map(|issue| cobs::issue::Issue::new(&id.into(), &issue, aliases));

        Ok::<_, Error>(issue)
    }

    fn comment_threads_by_issue_id(
        &self,
        rid: identity::RepoId,
        id: git::Oid,
    ) -> Result<Option<Vec<cobs::thread::Thread>>, Error> {
        let profile = self.profile();
        let repo = profile.storage.repository(rid)?;
        let issues = profile.issues(&repo)?;
        let issue = issues.get(&id.into())?;

        let aliases = &profile.aliases();
        let comments = issue.map(|issue| {
            issue
                .replies()
                // Filter out replies that aren't top level replies
                .filter(|c| {
                    let Some(oid) = c.1.reply_to() else {
                        return false;
                    };

                    oid == id
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
}

pub trait IssuesMut: Profile {
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
        cob_id: git::Oid,
        action: cobs::issue::Action,
        opts: cobs::CobOptions,
    ) -> Result<cobs::issue::Issue, Error> {
        let profile = self.profile();
        let mut node = Node::new(profile.socket());
        let repo = profile.storage.repository(rid)?;
        let signer = profile.signer()?;
        let aliases = profile.aliases();
        let mut issues = profile.issues_mut(&repo)?;
        let mut issue = issues.get_mut(&cob_id.into())?;

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
                    reply_to.unwrap_or(cob_id),
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
