use radicle::crypto::{ssh::Passphrase, PublicKey};

use crate::error::Error;

use super::traits::IdentityService;

#[derive(Debug, Clone)]
pub struct Service<I>
where
    I: IdentityService,
{
    auth: I,
}

impl<I> Service<I>
where
    I: IdentityService,
{
    pub fn check_agent(public_key: PublicKey) -> Result<(), Error> {
        match radicle::crypto::ssh::agent::Agent::connect() {
            Ok(mut agent) => {
                if agent.request_identities()?.contains(&public_key) {
                    Ok(())
                } else {
                    Err(Error::KeysNotRegistered)
                }
            }
            Err(e) if e.is_not_running() => Err(Error::AgentNotRunning)?,
            Err(e) => Err(e)?,
        }
    }
}

impl<I> Service<I>
where
    I: IdentityService,
{
    pub fn new(auth: I) -> Self {
        Self { auth }
    }
}

impl<I> IdentityService for Service<I>
where
    I: IdentityService,
{
    fn authenticate(&self, passphrase: Passphrase) -> Result<(), Error> {
        self.auth.authenticate(passphrase)
    }
}
