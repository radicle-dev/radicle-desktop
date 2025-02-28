use std::time;

use crossbeam_channel as chan;
use radicle::crypto::ssh::keystore::MemorySigner;
use radicle::identity::Did;
use radicle::node::{policy, Address, ConnectOptions, Handle};
use radicle::prelude::RepoId;
use radicle::Node;
use radicle_node::Runtime;
use radicle_types::error::Error;

use crate::AppState;

#[tauri::command]
pub(crate) fn stop_node(ctx: tauri::State<'_, AppState>) -> Result<(), Error> {
    let node = Node::new(ctx.profile.socket());
    node.shutdown()?;

    Ok(())
}

#[tauri::command]
pub(crate) async fn connect_node(
    ctx: tauri::State<'_, AppState>,
    from: Did,
    address: Address,
) -> Result<(), Error> {
    let mut node = Node::new(ctx.profile.socket());
    let _ = node.connect(
        from.into(),
        address,
        ConnectOptions {
            persistent: true,
            timeout: std::time::Duration::from_secs(120),
        },
    );
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    while node.session(from.into())?.is_none() {
        interval.tick().await;
    }

    Ok(())
}

#[tauri::command]
pub(crate) async fn start_node(
    ctx: tauri::State<'_, AppState>,
    signer: tauri::State<'_, MemorySigner>,
) -> Result<(), Error> {
    let profile = ctx.profile.clone();
    let node = Node::new(profile.socket());
    if node.is_running() {
        log::debug!("Node is already running.");
        return Ok(());
    }

    let mut config = radicle::profile::Config::load(&profile.home().config())?;
    config.node.connect.extend(config.preferred_seeds);

    if let Err(e) = radicle::io::set_file_limit(config.node.limits.max_open_files) {
        log::warn!(target: "node", "Unable to set process open file limit: {e}");
    }

    let (notify, signals) = chan::bounded(1);
    let _ = radicle_signals::install(notify);

    let signer = (*signer).clone();
    let home = profile.home().clone();

    tauri::async_runtime::spawn(async move {
        let _ = Runtime::init::<MemorySigner>(
            home,
            config.node.clone(),
            config.node.listen,
            signals,
            signer,
        )
        .unwrap()
        .run();
    });

    let node = Node::new(ctx.profile.socket());
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(60));

    while !node.is_running() {
        log::debug!("Still waiting for running");
        interval.tick().await;
    }

    Ok(())
}

#[tauri::command]
pub(crate) async fn fetch_repo(
    ctx: tauri::State<'_, AppState>,
    rid: RepoId,
    from: Did,
    timeout_seconds: u64,
) -> Result<(), Error> {
    let profile = &ctx.profile;
    let mut node = Node::new(profile.socket());

    profile.add_inventory(rid, &mut node)?;
    profile.seed(rid, policy::Scope::All, &mut node)?;
    node.fetch(rid, from.into(), time::Duration::from_secs(timeout_seconds))?;

    Ok(())
}
