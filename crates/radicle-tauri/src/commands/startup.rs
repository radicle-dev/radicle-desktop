use std::collections::BTreeMap;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use radicle::cob::cache::COBS_DB_FILE;
use radicle::identity::RepoId;
use radicle::node::{Handle, Node, NOTIFICATIONS_DB_FILE};
use radicle::storage::ReadStorage;

use radicle_types::config::Config;
use radicle_types::error::Error;
use radicle_types::traits::Profile;
use radicle_types::{domain, AppState};

pub struct Version {
    pub version: String,
    pub head: String,
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{} ({})", self.version, self.head))
    }
}

#[tauri::command]
pub(crate) fn version(app: AppHandle) -> Result<Version, Error> {
    let version = app
        .config()
        .version
        .clone()
        .expect("The build version has not been set.");
    pub const GIT_HEAD: &str = env!("GIT_HEAD");

    Ok(Version {
        version,
        head: GIT_HEAD.to_string(),
    })
}

#[tauri::command]
pub(crate) fn check_radicle_cli(ctx: tauri::State<AppState>) -> Result<(), Error> {
    let rad = ctx.profile().home().path().join("bin/rad");
    if !rad.try_exists()? {
        return Err(Error::RadicleNotInstalled);
    }

    Ok(())
}

#[tauri::command]
pub(crate) fn startup(app: AppHandle) -> Result<Config, Error> {
    let profile = radicle::Profile::load()?;
    let repositories = profile.storage.repositories()?;
    let public_key = profile.public_key;

    let inbox_db = radicle_types::outbound::sqlite::Sqlite::reader(
        profile.node().join(NOTIFICATIONS_DB_FILE),
    )?;
    let cob_db =
        radicle_types::outbound::sqlite::Sqlite::reader(profile.cobs().join(COBS_DB_FILE))?;

    let inbox_service = domain::inbox::service::Service::new(inbox_db);
    let patch_service = domain::patch::service::Service::new(cob_db);

    let node_handle = app.app_handle().clone();
    let sync_handle = app.app_handle().clone();
    let events_handle = app.app_handle().clone();

    let node = Node::new(profile.socket());
    let node_status = node.clone();

    let mut node_seeds = node.clone();

    app.manage(inbox_service);
    app.manage(patch_service);

    tauri::async_runtime::spawn(async move {
        loop {
            let _ = node_handle.emit("node_running", node_status.is_running());
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

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

    let state = AppState { profile };
    app.manage(state.clone());

    Ok(state.config())
}
