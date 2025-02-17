mod commands;

use radicle_types::AppState;

use commands::{auth, cob, diff, inbox, init, profile, repo, thread};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build());
    #[cfg(not(debug_assertions))]
    let builder = tauri::Builder::default();

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            init::startup,
            init::node_status_events,
            init::repo_sync_events,
            init::node_events,
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
