mod commands;

use tauri::{Emitter, Manager};

use radicle::cob::cache::COBS_DB_FILE;
use radicle::node::{Handle, NOTIFICATIONS_DB_FILE};
use radicle::Node;

use radicle_types::domain;
use radicle_types::error::Error;
use radicle_types::AppState;

use commands::{auth, cob, diff, inbox, profile, repo, thread};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build());
    #[cfg(not(debug_assertions))]
    let builder = tauri::Builder::default();

    builder
        .setup(|app| {
            let profile: radicle::Profile = match radicle::Profile::load() {
                Ok(profile) => Ok::<radicle::Profile, Error>(profile),
                Err(radicle::profile::Error::NotFound(path)) => Err(Error::WithHint {
                    err: anyhow::anyhow!("Radicle profile not found in '{}'.", path.display()),
                    hint: "To setup your radicle profile, run `rad auth`.",
                }),
                Err(e) => Err(Error::WithHint {
                    err: e.into(),
                    hint: "Could not load radicle profile",
                }),
            }?;

            let inbox_db = radicle_types::outbound::sqlite::Sqlite::reader(
                profile.node().join(NOTIFICATIONS_DB_FILE),
            )?;
            let inbox_service = domain::inbox::service::Service::new(inbox_db);

            let patch_db =
                radicle_types::outbound::sqlite::Sqlite::reader(profile.cobs().join(COBS_DB_FILE))?;
            let patch_service = domain::patch::service::Service::new(patch_db);

            let events_handler = app.handle().clone();
            let node_handler = app.handle().clone();

            let node = Node::new(profile.socket());
            let node_status = node.clone();

            app.manage(inbox_service);
            app.manage(patch_service);
            app.manage(AppState { profile });

            tauri::async_runtime::spawn(async move {
                loop {
                    let _ = node_handler.emit("node_running", node_status.is_running());
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            });

            tauri::async_runtime::spawn(async move {
                loop {
                    if node.is_running() {
                        log::debug!("node: spawned node event subscription.");
                        while let Ok(events) = node.subscribe(std::time::Duration::MAX) {
                            for event in events.into_iter().flatten() {
                                let _ = events_handler.emit("event", event);
                            }
                        }
                        log::debug!("node: event subscription loop has exited.");
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            auth::authenticate,
            repo::repo_count,
            repo::list_repos,
            repo::repo_by_id,
            repo::diff_stats,
            repo::list_commits,
            diff::get_diff,
            inbox::list_notifications,
            inbox::count_notifications_by_repo,
            inbox::clear_notifications,
            cob::get_embed,
            cob::save_embed_to_disk,
            cob::save_embed_by_path,
            cob::save_embed_by_clipboard,
            cob::save_embed_by_bytes,
            cob::issue::activity_by_issue,
            cob::issue::list_issues,
            cob::issue::issue_by_id,
            cob::issue::comment_threads_by_issue_id,
            cob::issue::create_issue,
            cob::issue::edit_issue,
            cob::patch::activity_by_patch,
            cob::patch::list_patches,
            cob::patch::patch_by_id,
            cob::patch::edit_patch,
            cob::patch::revisions_by_patch,
            cob::patch::revision_by_patch_and_id,
            thread::create_issue_comment,
            thread::create_patch_comment,
            profile::config,
            profile::alias,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
