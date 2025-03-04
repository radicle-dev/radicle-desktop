use radicle::crypto::ssh;

use crate::error::{self, Error};

use super::Profile;

pub trait Auth: Profile {
    fn authenticate(&self, passphrase: Option<String>) -> Result<(), Error> {
        let profile = self.profile();

        if !profile
            .keystore
            .is_encrypted()
            .map_err(error::crypto::CryptoError::Crypto)?
        {
            return Ok(());
        }
        match ssh::agent::Agent::connect() {
            Ok(mut agent) => {
                if agent
                    .request_identities()
                    .map_err(error::crypto::CryptoError::Agent)?
                    .contains(&profile.public_key)
                {
                    return Ok(());
                }
                let passphrase: Option<ssh::Passphrase> = passphrase.map(|p| p.into());
                profile
                    .keystore
                    .secret_key(passphrase.clone())
                    .map_err(error::crypto::CryptoError::Crypto)?;

                if let Some(pass) = passphrase {
                    register(&mut agent, &profile, pass)?;
                }

                Ok(())
            }
            Err(e) if e.is_not_running() => Err(error::auth::IdentityError::SSHAgentNotRunning)?,
            Err(e) => Err(error::crypto::CryptoError::Agent(e))?,
        }
    }
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
                error::auth::IdentityError::InvalidPassphrase
            } else {
                e.into()
            }
        })?
        .ok_or(error::auth::IdentityError::KeyNotFoundError)?;

    agent
        .register(&secret)
        .map_err(error::crypto::CryptoError::Agent)?;

    Ok(())
}
