mod commands;
mod error;
mod types;

use tauri::Emitter;
use tauri::Manager;

use radicle::node::Handle;
use radicle::Node;

use commands::{auth, cob, profile, repo, thread};

struct AppState {
    profile: radicle::Profile,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let builder = tauri::Builder::default().plugin(tauri_plugin_devtools::init());
    #[cfg(not(debug_assertions))]
    let builder = tauri::Builder::default();

    builder
        .setup(|app| {
            let profile: radicle::Profile = match radicle::Profile::load() {
                Ok(profile) => Ok::<radicle::Profile, error::Error>(profile),
                Err(radicle::profile::Error::NotFound(path)) => Err(error::Error::WithHint {
                    err: anyhow::anyhow!("Radicle profile not found in '{}'.", path.display()),
                    hint: "To setup your radicle profile, run `rad auth`.",
                }),
                Err(e) => Err(error::Error::WithHint {
                    err: e.into(),
                    hint: "Could not load radicle profile",
                }),
            }?;

            let events_handler = app.handle().clone();
            let node_handler = app.handle().clone();

            let node = Node::new(profile.socket());
            let node_status = node.clone();

            app.manage(AppState { profile });

            tauri::async_runtime::spawn(async move {
                loop {
                    let _ = node_handler.emit("node_running", node_status.is_running());
                    std::thread::sleep(std::time::Duration::from_secs(2));
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

                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            auth::authenticate,
            repo::list_repos,
            repo::repo_by_id,
            repo::diff_stats,
            cob::get_file_by_oid,
            cob::activity_by_id,
            cob::save_embed,
            cob::issue::list_issues,
            cob::issue::issue_by_id,
            cob::issue::create_issue,
            cob::issue::edit_issue,
            cob::patch::list_patches,
            cob::patch::patch_by_id,
            cob::patch::revisions_by_patch,
            cob::patch::revision_by_patch_and_id,
            cob::patch::create_draft_review,
            cob::patch::create_draft_review_comment,
            cob::patch::get_draft_review,
            cob::patch::edit_draft_review,
            cob::draft::publish_draft,
            thread::create_issue_comment,
            thread::create_patch_comment,
            profile::config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
