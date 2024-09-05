use crate::error::Error;
use crate::types::config::Config;
use crate::AppState;

/// Get active config.
#[tauri::command]
pub fn config(ctx: tauri::State<AppState>) -> Result<Config, Error> {
    let config = Config {
        public_key: ctx.profile.public_key,
        alias: ctx.profile.config.node.alias.clone(),
        seeding_policy: ctx.profile.config.node.seeding_policy,
    };

    Ok::<_, Error>(config)
}
