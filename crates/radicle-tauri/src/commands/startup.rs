use tauri::{AppHandle, Emitter, Manager};

use radicle::cob::cache::COBS_DB_FILE;
use radicle::node::{Handle, NOTIFICATIONS_DB_FILE, Node};

use radicle_types::config::{Config, Version};
use radicle_types::error::Error;
use radicle_types::traits::Profile;
use radicle_types::{AppState, domain};

#[tauri::command]
pub(crate) fn version(app: AppHandle) -> Result<Version, Error> {
    Ok(Version {
        version: app.config().version.clone().unwrap_or("unknown".into()),
        head: env!("GIT_HEAD").to_string(),
    })
}

#[tauri::command]
pub(crate) fn check_radicle_cli(ctx: tauri::State<AppState>) -> Result<(), Error> {
    // Where the official installer puts it, whether or not it ended up on
    // `PATH`.
    let rad = ctx.profile().home().path().join("bin").join("rad");
    if rad.exists() {
        return Ok(());
    }

    if radicle_types::binaries::rad().is_some() {
        return Ok(());
    }

    Err(Error::RadicleNotInstalled)
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
    let cobs_db =
        radicle_types::outbound::sqlite::Sqlite::reader(profile.cobs().join(COBS_DB_FILE))?;

    let inbox_service = domain::inbox::service::Service::new(inbox_db);
    let patch_service = domain::patch::service::Service::new(cobs_db.clone());
    let issue_service = domain::issue::service::Service::new(cobs_db);

    let node_handle = app.app_handle().clone();

    let node = Node::new(profile.home().socket_from_env());

    app.manage(inbox_service);
    app.manage(patch_service);
    app.manage(issue_service);

    tauri::async_runtime::spawn(async move {
        loop {
            let _ = node_handle.emit("node_running", node.is_running());
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    let state = AppState { profile };
    app.manage(state.clone());

    Ok(state.config())
}
