use radicle::node::NodeId;
use radicle_types::config::Config;
use radicle_types::traits::Profile;

use crate::AppState;

#[tauri::command]
pub fn config(ctx: tauri::State<AppState>) -> Config {
    ctx.config()
}

#[tauri::command]
pub fn alias(ctx: tauri::State<AppState>, nid: NodeId) -> Option<radicle::node::Alias> {
    ctx.alias(nid)
}
