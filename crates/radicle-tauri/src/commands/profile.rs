use radicle_types::config::Config;
use radicle_types::traits::Profile;

use crate::AppState;

#[tauri::command]
pub fn config(ctx: tauri::State<AppState>) -> Config {
    ctx.config()
}
