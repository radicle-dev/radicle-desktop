use radicle::node::{AliasStore, NodeId};
use radicle::Profile;
use radicle_types::config::Config;

#[tauri::command]
pub fn config(profile: tauri::State<Profile>) -> Config {
    Config::get(&profile)
}

#[tauri::command]
pub fn alias(profile: tauri::State<Profile>, nid: NodeId) -> Option<radicle::node::Alias> {
    profile.alias(&nid)
}
