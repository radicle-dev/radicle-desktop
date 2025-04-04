use radicle::node::{AliasStore, NodeId};

use radicle_types::config::Config;

#[tauri::command]
pub fn config(profile: tauri::State<radicle::Profile>) -> Config {
    Config::get(&profile)
}

#[tauri::command]
pub fn alias(profile: tauri::State<radicle::Profile>, nid: NodeId) -> Option<radicle::node::Alias> {
    profile.alias(&nid)
}
