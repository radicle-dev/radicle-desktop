mod commands;

use tauri::Emitter;
use tauri::Manager;

use radicle::node::Handle;
use radicle::Node;
use radicle_types::error::Error;
use radicle_types::traits::auth::Auth;
use radicle_types::traits::cobs::Cobs;
use radicle_types::traits::issue::{Issues, IssuesMut};
use radicle_types::traits::patch::{Patches, PatchesMut};
use radicle_types::traits::repo::Repo;
use radicle_types::traits::thread::Thread;
use radicle_types::traits::Profile;

use commands::{auth, cob, diff, profile, repo, thread};

struct AppState {
    profile: radicle::Profile,
}

impl Auth for AppState {}
impl Repo for AppState {}
impl Thread for AppState {}
impl Cobs for AppState {}
impl Issues for AppState {}
impl IssuesMut for AppState {}
impl Patches for AppState {}
impl PatchesMut for AppState {}
impl Profile for AppState {
    fn profile(&self) -> radicle::Profile {
        self.profile.clone()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let builder = tauri::Builder::default().plugin(tauri_plugin_log::Builder::new().build());
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

            let events_handler = app.handle().clone();
            let node_handler = app.handle().clone();

            let node = Node::new(profile.socket());
            let node_status = node.clone();

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
            repo::list_repos,
            repo::repo_by_id,
            repo::diff_stats,
            diff::get_diff,
            cob::get_file_by_oid,
            cob::activity_by_id,
            cob::save_embed,
            cob::issue::list_issues,
            cob::issue::issue_by_id,
            cob::issue::comment_threads_by_issue_id,
            cob::issue::create_issue,
            cob::issue::edit_issue,
            cob::patch::list_patches,
            cob::patch::patch_by_id,
            cob::patch::edit_patch,
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
