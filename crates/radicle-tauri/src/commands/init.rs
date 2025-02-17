use std::collections::BTreeMap;

use radicle::cob::cache::COBS_DB_FILE;
use radicle::identity::RepoId;
use radicle::node::{Handle, Node, NOTIFICATIONS_DB_FILE};
use radicle::storage::ReadStorage;
use tauri::{AppHandle, Emitter, Manager};

use radicle_types::config::Config;
use radicle_types::error::Error;
use radicle_types::traits::Profile;
use radicle_types::{domain, AppState};

#[tauri::command]
pub(crate) fn startup(app: AppHandle) -> Result<Config, Error> {
    let profile = radicle::Profile::load()?;

    let inbox_db = radicle_types::outbound::sqlite::Sqlite::reader(
        profile.node().join(NOTIFICATIONS_DB_FILE),
    )?;
    let cob_db =
        radicle_types::outbound::sqlite::Sqlite::reader(profile.cobs().join(COBS_DB_FILE))?;

    let inbox_service = domain::inbox::service::Service::new(inbox_db);
    let patch_service = domain::patch::service::Service::new(cob_db);

    app.manage(inbox_service);
    app.manage(patch_service);

    let state = AppState { profile };
    app.manage(state.clone());

    Ok(state.config())
}

#[tauri::command]
pub(crate) fn node_status_events(app: AppHandle, ctx: tauri::State<AppState>) -> Result<(), Error> {
    let app_handle = app.clone();

    let node = Node::new(ctx.profile.socket());
    let node_status = node.clone();

    tauri::async_runtime::spawn(async move {
        loop {
            let _ = app_handle.emit("node_running", node_status.is_running());
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    Ok(())
}

#[tauri::command]
pub(crate) fn repo_sync_events(app: AppHandle, ctx: tauri::State<AppState>) -> Result<(), Error> {
    let profile = &ctx.profile;
    let repositories = profile.storage.repositories()?;

    let app_handle = app.clone();
    let public_key = profile.public_key;

    let node = Node::new(profile.socket());
    let mut node_seeds = node.clone();

    tauri::async_runtime::spawn(async move {
        loop {
            let mut sync_status =
                BTreeMap::<RepoId, Option<radicle_types::cobs::repo::SyncStatus>>::new();
            for repo in &repositories {
                if let Ok(seeds) = node_seeds.seeds(repo.rid).map(Into::<Vec<_>>::into) {
                    if let Some(status) =
                        seeds
                            .iter()
                            .find_map(|radicle::node::Seed { nid, sync, .. }| {
                                (*nid == public_key).then_some(sync.clone())
                            })
                    {
                        sync_status.insert(repo.rid, status.map(Into::into));
                    } else {
                        // The local node wasn't found in the seed nodes table.
                        sync_status.insert(repo.rid, None);
                    }
                }
            }
            let _ = app_handle.emit("sync_status", sync_status);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });

    Ok(())
}

#[tauri::command]
pub(crate) fn node_events(app: AppHandle, ctx: tauri::State<AppState>) -> Result<(), Error> {
    let app_handle = app.clone();
    let node = Node::new(ctx.profile.socket());

    tauri::async_runtime::spawn(async move {
        loop {
            if node.is_running() {
                log::debug!("node: spawned node event subscription.");
                while let Ok(events) = node.subscribe(std::time::Duration::MAX) {
                    for event in events.into_iter().flatten() {
                        let _ = app_handle.emit("event", event);
                    }
                }
                log::debug!("node: event subscription loop has exited.");
            }

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    Ok(())
}
