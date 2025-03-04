use std::str::FromStr;

use radicle::crypto::ssh;
use radicle::node::Alias;
use radicle::profile::env;
use radicle_types::error::auth::IdentityError;
use radicle_types::error::crypto::CryptoError;
use radicle_types::error::Error;
use radicle_types::traits::auth::register;
use radicle_types::traits::auth::Auth;
use tauri::{AppHandle, Manager};

use crate::AppState;

#[tauri::command]
pub fn authenticate(
    app: AppHandle,
    ctx: tauri::State<AppState>,
    passphrase: Option<String>,
) -> Result<(), Error> {
    let signer = ssh::keystore::MemorySigner::load(
        &ctx.profile.keystore,
        passphrase.clone().map(Into::into),
    )
    .map_err(IdentityError::MemorySignerError)?;
    app.manage(signer);

    ctx.authenticate(passphrase)
}

#[tauri::command]
pub(crate) fn init(alias: String, passphrase: String) -> Result<(), Error> {
    let home = radicle::profile::home()?;
    let alias = Alias::from_str(&alias).map_err(IdentityError::AliasError)?;
    if passphrase.is_empty() {
        return Err(IdentityError::MissingPassphrase.into());
    }
    let profile =
        radicle::Profile::init(home, alias, Some(passphrase.clone().into()), env::seed())?;

    match ssh::agent::Agent::connect() {
        Ok(mut agent) => register(&mut agent, &profile, passphrase.clone().into())?,
        Err(e) if e.is_not_running() => return Err(IdentityError::SSHAgentNotRunning.into()),
        Err(e) => Err(CryptoError::Agent(e))?,
    }

    Ok(())
}
