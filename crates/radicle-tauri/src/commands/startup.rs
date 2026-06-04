use tauri::{AppHandle, Emitter, Manager};

use radicle::cob::cache::COBS_DB_FILE;
use radicle::node::{Handle, NOTIFICATIONS_DB_FILE, Node};

use radicle_artifact_client::tokio::Client;

use radicle_types::config::{Config, Version};
use radicle_types::error::Error;
use radicle_types::traits::Profile;
use radicle_types::{AppState, ArtifactClient, domain};

#[tauri::command]
pub(crate) fn version(app: AppHandle) -> Result<Version, Error> {
    Ok(Version {
        version: app.config().version.clone().unwrap_or("unknown".into()),
        head: env!("GIT_HEAD").to_string(),
    })
}

/// Check whether a binary can be found in the most common paths on Unix-like systems.
/// We don't bother checking the `$PATH` variable, as we're only looking for very standard tools
/// and prefer not to make this too complex.
#[cfg(unix)]
fn exists(cmd: &str) -> bool {
    // Some common paths where system-installed binaries are found.
    const PATHS: &[&str] = &["/usr/local/bin", "/usr/bin", "/bin"];

    for dir in PATHS {
        if std::path::Path::new(dir).join(cmd).exists() {
            return true;
        }
    }
    false
}

/// Check whether a binary can be found on `$PATH`.
/// See:
///  - <https://devblogs.microsoft.com/scripting/weekend-scripter-where-exethe-what-why-and-how/>
///  - <https://learn.microsoft.com/windows-server/administration/windows-commands/where>
#[cfg(windows)]
fn exists(cmd: &str) -> bool {
    use std::os::windows::process::CommandExt;

    // See <https://learn.microsoft.com/windows/win32/procthread/process-creation-flags#flags>.
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    std::process::Command::new("where.exe")
        .arg("/q")
        .arg("$PATH:".to_owned() + cmd)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|output| output.status.success())
        .unwrap_or_default()
}

#[tauri::command]
pub(crate) fn check_radicle_cli(ctx: tauri::State<AppState>) -> Result<(), Error> {
    let rad = ctx.profile().home().path().join("bin").join("rad");
    if rad.exists() {
        return Ok(());
    }

    if exists("rad") {
        return Ok(());
    }

    Err(Error::RadicleNotInstalled)
}

#[tauri::command]
pub(crate) async fn startup(app: AppHandle) -> Result<Config, Error> {
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

    // Client for the external rad-artifact node. Only wraps a socket path,
    // so this never blocks startup — seeding/download calls fail with
    // `ArtifactNodeNotRunning` if the node isn't up.
    let artifact_client = Client::new(Client::default_socket(home.path()));

    let node_handle = app.app_handle().clone();
    let node = Node::new(profile.home().socket_from_env());

    app.manage(inbox_service);
    app.manage(patch_service);

    tauri::async_runtime::spawn(async move {
        loop {
            let _ = node_handle.emit("node_running", node.is_running());
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    // Mirror the radicle-node liveness loop for the artifact node so the UI
    // can disable seeding/download and show an indicator when it's down.
    let artifact_node_handle = app.app_handle().clone();
    let artifact_probe = artifact_client.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let _ = artifact_node_handle
                .emit("artifact_node_running", artifact_probe.is_running().await);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    let state = AppState { profile };
    app.manage(state.clone());
    app.manage(ArtifactClient {
        client: artifact_client,
    });

    Ok(state.config())
}
