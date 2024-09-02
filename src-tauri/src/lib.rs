mod auth;
mod error;

use auth::authenticate;
use tauri::Manager;

struct AppState {
    profile: radicle::Profile,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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

            app.manage(AppState { profile });

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![authenticate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
