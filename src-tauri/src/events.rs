use std::sync::{atomic::AtomicBool, Arc};

use radicle::{node::Handle, Node, Profile};
use tauri::{AppHandle, Emitter};

use crate::error::Error;

pub async fn subscribe_events(
    handle: &AppHandle,
    profile: Profile,
    existing_events_thread: Arc<AtomicBool>,
) -> Result<(), Error> {
    let event_handler = handle.clone();
    let node = Node::new(profile.socket());
    if !node.is_running() {
        event_handler.emit("node_status", "stopped")?;
        log::debug!("node: not subscribing to events due to stopped node.");
        return Ok(());
    };
    event_handler.emit("node_status", "running")?;

    if existing_events_thread.load(std::sync::atomic::Ordering::SeqCst) {
        log::debug!("node: not subscribing to events due to a running subscription.");
        return Ok(());
    } else {
        let join_handle = tauri::async_runtime::spawn(async move {
            log::debug!("node: spawned node event subscription.");
            while let Ok(events) = node.subscribe(std::time::Duration::MAX) {
                existing_events_thread.store(true, std::sync::atomic::Ordering::SeqCst);
                for event in events.into_iter().flatten() {
                    let _ = event_handler.emit("event", event);
                }
            }
            existing_events_thread.store(false, std::sync::atomic::Ordering::SeqCst);
            log::debug!("node: event subscription loop has exited.");
        });

        join_handle.await?;
    }

    Ok(())
}
