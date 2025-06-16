use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use radicle::cob::cache::COBS_DB_FILE;
use radicle::node::{Handle, Node, NOTIFICATIONS_DB_FILE};

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
    Ok(Version {
        version: app.config().version.clone().unwrap_or("unknown".into()),
        head: env!("GIT_HEAD").to_string(),
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
    let home = profile.home();

    let cobs_cache = radicle::cob::cache::Store::open(home.cobs().join(COBS_DB_FILE))?;
    cobs_cache.check_version()?;

    let inbox_db = radicle_types::outbound::sqlite::Sqlite::reader(
        profile.node().join(NOTIFICATIONS_DB_FILE),
    )?;
    let cob_db =
        radicle_types::outbound::sqlite::Sqlite::reader(profile.cobs().join(COBS_DB_FILE))?;

    let inbox_service = domain::inbox::service::Service::new(inbox_db);
    let patch_service = domain::patch::service::Service::new(cob_db);

    let node_handle = app.app_handle().clone();

    let node = Node::new(profile.socket());
    let node_status = node.clone();

    app.manage(inbox_service);
    app.manage(patch_service);

    tauri::async_runtime::spawn(async move {
        loop {
            let _ = node_handle.emit("node_running", node_status.is_running());
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    let state = AppState { profile };
    app.manage(state.clone());

    Ok(state.config())
}
