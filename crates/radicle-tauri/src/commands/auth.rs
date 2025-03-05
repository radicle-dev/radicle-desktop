use std::str::FromStr;

use radicle::crypto::ssh;
use radicle::node::Alias;
use radicle::profile::env;
use radicle_types::error::auth::IdentityError;
use radicle_types::error::crypto::CryptoError;
use radicle_types::error::Error;

use crate::AppState;

#[tauri::command]
pub fn authenticate(ctx: tauri::State<AppState>, passphrase: Option<String>) -> Result<(), Error> {
    let profile = ctx.profile.clone();

    if !profile
        .keystore
        .is_encrypted()
        .map_err(CryptoError::Crypto)?
    {
        return Ok(());
    }
    match ssh::agent::Agent::connect() {
        Ok(mut agent) => {
            if agent
                .request_identities()
                .map_err(CryptoError::Agent)?
                .contains(&profile.public_key)
            {
                return Ok(());
            }
            let passphrase: Option<ssh::Passphrase> = passphrase.map(|p| p.into());
            profile
                .keystore
                .secret_key(passphrase.clone())
                .map_err(IdentityError::Crypto)?;

            if let Some(pass) = passphrase {
                register(&mut agent, &profile, pass)?;
            }

            Ok(())
        }
        Err(e) if e.is_not_running() => Err(IdentityError::SSHAgentNotRunning)?,
        Err(e) => Err(CryptoError::Agent(e))?,
    }
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
                IdentityError::InvalidPassphrase
            } else {
                e.into()
            }
        })?
        .ok_or(IdentityError::KeyNotFoundError)?;

    agent.register(&secret).map_err(CryptoError::Agent)?;

    Ok(())
}
