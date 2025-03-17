use std::str::FromStr;

use radicle::crypto::ssh::{self, Passphrase};
use radicle::node::Alias;
use radicle::profile::env;
use radicle_types::error::Error;

use crate::AppState;

#[tauri::command]
pub fn authenticate(
    ctx: tauri::State<AppState>,
    passphrase: Option<Passphrase>,
) -> Result<(), Error> {
    let profile = &ctx.profile;
    if !profile.keystore.is_encrypted()? {
        return Ok(());
    }
    match ssh::agent::Agent::connect() {
        Ok(mut agent) => {
            if agent.request_identities()?.contains(&profile.public_key) {
                return Ok(());
            }

            match passphrase {
                Some(passphrase) => {
                    profile.keystore.secret_key(Some(passphrase.clone()))?;
                    register(&mut agent, profile, passphrase)
                }
                None => Err(Error::Crypto(
                    radicle::crypto::ssh::keystore::Error::PassphraseMissing,
                )),
            }
        }
        Err(e) if e.is_not_running() => Err(Error::AgentNotRunning)?,
        Err(e) => Err(e)?,
    }
}

#[tauri::command]
pub(crate) fn init(alias: String, passphrase: Passphrase) -> Result<(), Error> {
    let home = radicle::profile::home()?;
    let alias = Alias::from_str(&alias)?;

    if passphrase.is_empty() {
        return Err(Error::Crypto(
            radicle::crypto::ssh::keystore::Error::PassphraseMissing,
        ));
    }
    let profile = radicle::Profile::init(home, alias, Some(passphrase.clone()), env::seed())?;
    match ssh::agent::Agent::connect() {
        Ok(mut agent) => register(&mut agent, &profile, passphrase.clone())?,
        Err(e) if e.is_not_running() => return Err(Error::AgentNotRunning),
        Err(e) => Err(e)?,
    }

    Ok(())
}

pub fn register(
    agent: &mut ssh::agent::Agent,
    profile: &radicle::Profile,
    passphrase: ssh::Passphrase,
) -> Result<(), Error> {
    let secret = profile
        .keystore
        .secret_key(Some(passphrase))
        .map_err(|e| {
            if e.is_crypto_err() {
                Error::Crypto(radicle::crypto::ssh::keystore::Error::Ssh(
                    ssh_key::Error::Crypto,
                ))
            } else {
                e.into()
            }
        })?
        .ok_or(Error::Crypto(radicle::crypto::ssh::keystore::Error::Ssh(
            ssh_key::Error::Crypto,
        )))?;

    agent.register(&secret)?;

    Ok(())
}
