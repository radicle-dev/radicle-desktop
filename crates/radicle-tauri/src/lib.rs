mod commands;

use radicle_types::AppState;

use commands::{auth, cob, diff, inbox, profile, repo, startup, thread};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        );
    #[cfg(not(debug_assertions))]
    let builder = tauri::Builder::default();

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            auth::authenticate,
            auth::init,
            cob::get_embed,
            cob::issue::activity_by_issue,
            cob::issue::comment_threads_by_issue_id,
            cob::issue::create_issue,
            cob::issue::edit_issue,
            cob::issue::issue_by_id,
            cob::issue::list_issues,
            cob::issue::rebuild_issue_cache,
            cob::patch::activity_by_patch,
            cob::patch::edit_patch,
            cob::patch::list_patches,
            cob::patch::patch_by_id,
            cob::patch::edit_patch,
            cob::patch::rebuild_patch_cache,
            cob::patch::review_by_patch_and_revision_and_id,
            cob::patch::revisions_by_patch,
            cob::patch::revision_by_patch_and_id,
            cob::patch::revisions_by_patch,
            cob::save_embed_by_bytes,
            cob::save_embed_by_clipboard,
            cob::save_embed_by_path,
            cob::save_embed_to_disk,
            diff::get_diff,
            inbox::clear_notifications,
            inbox::notification_count,
            inbox::list_notifications,
            profile::alias,
            profile::config,
            repo::create_repo,
            repo::diff_stats,
            repo::list_commits,
            repo::list_repos,
            repo::repo_by_id,
            repo::repo_count,
            repo::repo_readme,
            repo::seed,
            repo::seeded_not_replicated,
            repo::unseed,
            startup::startup,
            startup::version,
            startup::check_radicle_cli,
            thread::create_issue_comment,
            thread::create_patch_comment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
