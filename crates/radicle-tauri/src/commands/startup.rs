use std::collections::BTreeMap;

use radicle::cob::cache::COBS_DB_FILE;
use radicle::identity::RepoId;
use radicle::node::{Handle, Node, NOTIFICATIONS_DB_FILE};
use radicle::storage::ReadStorage;
use radicle::Profile;
use radicle_types::outbound::radicle::Radicle;
use radicle_types::outbound::sqlite::Sqlite;
use tauri::{AppHandle, Emitter, Manager};

use radicle_types::config::Config;
use radicle_types::domain;
use radicle_types::error::Error;

#[tauri::command]
pub(crate) fn load_profile(app: AppHandle) -> Result<Config, Error> {
    let profile = radicle::Profile::load()?;
    app.manage(profile.clone());

    Ok(Config::get(&profile))
}

#[tauri::command]
pub(crate) fn create_services(app: AppHandle, profile: tauri::State<Profile>) -> Result<(), Error> {
    let inbox_db = Sqlite::reader(profile.node().join(NOTIFICATIONS_DB_FILE))?;
    let cob_db = Sqlite::reader(profile.cobs().join(COBS_DB_FILE))?;
    let radicle = Radicle::new((*profile).clone());

    let inbox_service = domain::inbox::service::Service::new(inbox_db);
    let repo_service = domain::repo::service::Service::new(radicle.clone(), cob_db);
    let identity_service = domain::identity::service::Service::new(radicle);

    app.manage(inbox_service);
    app.manage(repo_service);
    app.manage(identity_service);

    Ok(())
}

#[tauri::command]
pub(crate) fn create_event_emitters(
    app: AppHandle,
    profile: tauri::State<Profile>,
) -> Result<(), Error> {
    let node_handle = app.app_handle().clone();
    let sync_handle = app.app_handle().clone();
    let events_handle = app.app_handle().clone();

    let node = Node::new(profile.socket());
    let node_status = node.clone();

    let mut node_seeds = node.clone();

    let repositories = profile.storage.repositories()?;
    let public_key = profile.public_key;

    tauri::async_runtime::spawn(async move {
        loop {
            let _ = node_handle.emit("node_running", node_status.is_running());
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    tauri::async_runtime::spawn(async move {
        loop {
            let mut sync_status = BTreeMap::<
                RepoId,
                Option<radicle_types::domain::repo::models::repo::SyncStatus>,
            >::new();
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
            let _ = sync_handle.emit("sync_status", sync_status);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });

    tauri::async_runtime::spawn(async move {
        loop {
            if node.is_running() {
                log::info!("node: spawned node event subscription.");
                while let Ok(events) = node.subscribe(std::time::Duration::MAX) {
                    for event in events.into_iter().flatten() {
                        let _ = events_handle.emit("event", event);
                    }
                }
                log::info!("node: event subscription loop has exited.");
            }

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    Ok(())
}
