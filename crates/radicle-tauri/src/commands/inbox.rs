use std::collections::BTreeMap;

use radicle::identity;
use radicle::issue::cache::Issues;
use radicle::node;
use radicle::patch::cache::Patches;
use radicle::storage::{ReadRepository, ReadStorage};

use radicle_types::cobs::PaginatedQuery;
use radicle_types::domain::inbox::models::notification::{self, RepoGroupByItem};
use radicle_types::domain::inbox::service::Service;
use radicle_types::domain::inbox::traits::InboxService;
use radicle_types::error::Error;
use radicle_types::outbound::sqlite::Sqlite;
use radicle_types::AppState;

#[tauri::command]
pub fn list_notifications(
    ctx: tauri::State<AppState>,
    sqlite_service: tauri::State<Service<Sqlite>>,
    params: notification::RepoGroupParams,
) -> Result<PaginatedQuery<RepoGroupByItem>, Error> {
    let profile = &ctx.profile;
    let aliases = profile.aliases();
    let cursor = params.skip.unwrap_or(0);
    let take = params.take.unwrap_or(20);

    let all = sqlite_service.repo_group(params.clone())?;
    let more = cursor + take < all.len();
    let repo = profile.storage.repository(params.repo)?;
    let patches = profile.patches(&repo)?;
    let issues = profile.issues(&repo)?;

    let content = all
        .into_iter()
        .skip(cursor)
        .take(take)
        .map(|(qualified, n)| {
            let items = n
                .into_iter()
                .filter_map(|s| {
                    let update: notification::RefUpdate =
                        (qualified.clone().into_refstring(), s.new, s.old).into();
                    let update: radicle::storage::RefUpdate = update.into();
                    let kind =
                        node::notifications::NotificationKind::try_from(qualified.clone()).ok()?;

                    match kind {
                        node::notifications::NotificationKind::Cob { ref typed_id } => {
                            if typed_id.is_patch() {
                                let actions = notification::actions(
                                    typed_id.type_name.clone(),
                                    typed_id.id,
                                    update.old(),
                                    update.new(),
                                    &repo,
                                    &aliases,
                                )
                                .unwrap_or_default();

                                match patches.get(&typed_id.id) {
                                    Ok(Some(p)) => Some(notification::NotificationItem::Patch(
                                        notification::Patch {
                                            row_id: s.row_id,
                                            id: typed_id.id,
                                            update: update.into(),
                                            timestamp: s.timestamp,
                                            title: p.title().to_string(),
                                            status: (p.state().clone()).into(),
                                            actions,
                                        },
                                    )),
                                    Ok(None) => {
                                        log::error!("No patch found");
                                        None
                                    }
                                    Err(e) => {
                                        log::error!("{}", e);
                                        None
                                    }
                                }
                            } else if typed_id.is_issue() {
                                let actions = notification::actions(
                                    typed_id.type_name.clone(),
                                    typed_id.id,
                                    update.old(),
                                    update.new(),
                                    &repo,
                                    &aliases,
                                )
                                .unwrap_or_default();

                                match issues.get(&typed_id.id) {
                                    Ok(Some(i)) => Some(notification::NotificationItem::Issue(
                                        notification::Issue {
                                            row_id: s.row_id,
                                            id: typed_id.id,
                                            update: update.into(),
                                            timestamp: s.timestamp,
                                            title: i.title().to_string(),
                                            status: (*i.state()).into(),
                                            actions,
                                        },
                                    )),
                                    Ok(None) => {
                                        log::error!("No issue found");
                                        None
                                    }
                                    Err(e) => {
                                        log::error!("{}", e);
                                        None
                                    }
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                })
                .collect::<Vec<_>>();

            (qualified, items)
        })
        .filter(|(_, v)| !v.is_empty())
        .collect::<RepoGroupByItem>();

    Ok(PaginatedQuery {
        cursor,
        more,
        content,
    })
}

#[tauri::command]
pub fn count_notifications_by_repo(
    ctx: tauri::State<AppState>,
    inbox: tauri::State<Service<Sqlite>>,
) -> Result<BTreeMap<identity::RepoId, notification::NotificationCount>, Error> {
    let profile = &ctx.profile;
    let result = inbox
        .counts_by_repo()?
        .filter_map(|s| {
            let (rid, count) = s.ok()?;
            let repo = profile.storage.repository(rid).ok()?;
            let identity::DocAt { doc, .. } = repo.identity_doc().ok()?;
            let project = doc.project().ok()?;

            Some((
                rid,
                notification::NotificationCount {
                    rid,
                    name: project.name().to_string(),
                    count,
                },
            ))
        })
        .collect::<BTreeMap<identity::RepoId, notification::NotificationCount>>();

    Ok(result)
}

#[tauri::command]
pub fn clear_notifications(
    ctx: tauri::State<AppState>,
    params: notification::SetStatusNotifications,
) -> Result<(), Error> {
    let profile = &ctx.profile;
    let mut notifications = profile.notifications_mut()?;
    match params {
        notification::SetStatusNotifications::Ids(ids) => notifications.clear(&ids)?,
        notification::SetStatusNotifications::Repo(repo) => notifications.clear_by_repo(&repo)?,
        notification::SetStatusNotifications::All => notifications.clear_all()?,
    };

    Ok(())
}
