use std::str::FromStr;

use radicle::crypto::ssh::Passphrase;
use radicle::node::Alias;

use outbound::radicle::Radicle;

pub mod config;
pub mod domain;
pub mod error;
pub mod outbound;
pub mod test;

pub fn init(alias: String, passphrase: Passphrase) -> Result<(), error::Error> {
    let home = radicle::profile::home()?;
    let alias = Alias::from_str(&alias)?;

    if passphrase.is_empty() {
        return Err(error::Error::Crypto(
            radicle::crypto::ssh::keystore::Error::PassphraseMissing,
        ));
    }
    let profile = radicle::Profile::init(
        home,
        alias,
        Some(passphrase.clone()),
        radicle::profile::env::seed(),
    )?;
    match radicle::crypto::ssh::agent::Agent::connect() {
        Ok(mut agent) => Radicle::register(&mut agent, &profile, passphrase.clone())?,
        Err(e) if e.is_not_running() => return Err(error::Error::AgentNotRunning),
        Err(e) => Err(e)?,
    }

    Ok(())
}
