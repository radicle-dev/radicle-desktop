use std::time;

use crossbeam_channel as chan;
use radicle::crypto::ssh::keystore::MemorySigner;
use radicle::identity::Did;
use radicle::node::{Address, ConnectOptions, ConnectResult, Handle};
use radicle::Node;
use radicle_node::Runtime;
use radicle_types as types;
use radicle_types::error::node::NodeError;
use radicle_types::error::Error;

use crate::AppState;

/// How long to wait for the node to fetch a repo from the network.
pub const NODE_FETCH_TIMEOUT: time::Duration = time::Duration::from_secs(9);
/// How long to wait for the node to respond to a command.
pub const NODE_COMMAND_TIMEOUT: time::Duration = time::Duration::from_millis(500);
/// How long to wait for the node to start before returning an error.
pub const NODE_START_TIMEOUT: time::Duration = time::Duration::from_secs(6);

#[tauri::command]
pub(crate) fn stop_node(ctx: tauri::State<'_, AppState>) -> Result<(), Error> {
    let node = Node::new(ctx.profile.socket());
    node.shutdown()
        .map_err(types::error::node::NodeError::Node)?;

    Ok(())
}

#[tauri::command]
pub(crate) async fn connect_node(
    ctx: tauri::State<'_, AppState>,
    from: Did,
    address: Address,
) -> Result<(), Error> {
    let mut node = Node::new(ctx.profile.socket());
    match node.connect(
        from.into(),
        address,
        ConnectOptions {
            persistent: true,
            timeout: NODE_COMMAND_TIMEOUT,
        },
    ) {
        Ok(ConnectResult::Connected) => Ok(()),
        _ => Err(NodeError::ConnectError.into()),
    }
}

#[tauri::command]
pub(crate) async fn start_node(
    ctx: tauri::State<'_, AppState>,
    signer: tauri::State<'_, MemorySigner>,
) -> Result<(), Error> {
    let profile = ctx.profile.clone();
    let node = Node::new(profile.socket());
    if node.is_running() {
        log::debug!(target: "node", "Node is already running.");
        return Ok(());
    }

    let mut config = radicle::profile::Config::load(&profile.home.config())?;
    config.node.connect.extend(config.preferred_seeds);

    if let Err(e) = radicle::io::set_file_limit(config.node.limits.max_open_files) {
        log::warn!(target: "node", "Unable to set process open file limit: {e}");
    }

    let (notify, signals) = chan::bounded(1);
    if let Err(e) = radicle_signals::install(notify) {
        log::warn!(target: "node", "Unable to install signal handlers: {e}");
    }

    let signer = (*signer).clone();
    let started = std::time::Instant::now();

    tauri::async_runtime::spawn(async {
        match Runtime::init::<MemorySigner>(
            profile.home,
            config.node.clone(),
            config.node.listen,
            signals,
            signer,
        )
        .and_then(|runtime| runtime.run())
        {
            Ok(()) => (),
            Err(e) => log::error!(target: "node", "node spawning failed: {}", e),
        }
    });

    loop {
        if node.is_running() {
            log::debug!(target: "node", "Node started");

            break;
        } else if started.elapsed() >= NODE_START_TIMEOUT {
            return Err(NodeError::StartError.into());
        }

        tokio::time::sleep(NODE_COMMAND_TIMEOUT).await;
    }

    Ok(())
}
