use anyhow::anyhow;
use radicle::crypto::ssh;
use radicle::node::Alias;
use radicle::profile::env;
use radicle_types::traits::auth::Auth;
use radicle_types::{error::Error, traits::auth::register};
use tauri::{AppHandle, Manager};

use crate::AppState;

#[tauri::command]
pub fn authenticate(
    app: AppHandle,
    ctx: tauri::State<AppState>,
    passphrase: Option<String>,
) -> Result<(), Error> {
    // We store the passphrase in the tauri state so we can construct easily new `MemorySigners` from the `Keystore`
    // We need the `MemorySigner` to run a node.
    // TODO: Improving security by encrypting the passphrase at rest.
    let signer = ssh::keystore::MemorySigner::load(
        &ctx.profile.keystore,
        passphrase.clone().map(Into::into),
    )?;
    app.manage(signer);

    ctx.authenticate(passphrase)
}

#[tauri::command]
pub(crate) fn init(alias: String, passphrase: String) -> Result<(), Error> {
    let home = radicle::profile::home()?;
    let profile = radicle::Profile::init(
        home,
        Alias::new(alias),
        Some(passphrase.clone().into()),
        env::seed(),
    )?;

    match ssh::agent::Agent::connect() {
            Ok(mut agent) => register(&mut agent, &profile, passphrase.clone().into())?,
            Err(e) if e.is_not_running() => Err(Error::WithHint { err: anyhow!("SSH Agent is not running"), hint: "For now we require the user to have an ssh agent running, since we don't have passphrase inputs yet." })?, 
            Err(e) => Err(e)?,
        }

    Ok(())
}
