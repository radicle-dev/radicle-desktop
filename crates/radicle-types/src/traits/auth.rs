use anyhow::anyhow;

use radicle::crypto::ssh;

use crate::error::Error;

use super::Profile;

pub trait Auth: Profile {
    fn authenticate(&self, passphrase: Option<String>) -> Result<(), Error> {
        let profile = self.profile();

        if !profile.keystore.is_encrypted()? {
            return Ok(());
        }
        match ssh::agent::Agent::connect() {
        Ok(mut agent) => {
            if agent.request_identities()?.contains(&profile.public_key) {
               return Ok(());
            }
            let passphrase: Option<ssh::Passphrase> = passphrase.map(|p| p.into());
            profile.keystore.secret_key(passphrase.clone())?;

            if let Some(pass) = passphrase {
                register(&mut agent, &profile, pass)?;
            }

            Ok(())
        },
        Err(e) if e.is_not_running() => Err(Error::WithHint { err: anyhow!("SSH Agent is not running"), hint: "For now we require the user to have an ssh agent running, since we don't have passphrase inputs yet." })?, 
        Err(e) => Err(e)?,
      }
    }
}

pub fn register(
    agent: &mut ssh::agent::Agent,
    profile: &radicle::Profile,
    passphrase: ssh::Passphrase,
) -> anyhow::Result<()> {
    let secret = profile
        .keystore
        .secret_key(Some(passphrase))
        .map_err(|e| {
            if e.is_crypto_err() {
                anyhow!("could not decrypt secret key: invalid passphrase")
            } else {
                e.into()
            }
        })?
        .ok_or_else(|| anyhow!("Key not found in {:?}", profile.keystore.path()))?;

    agent.register(&secret)?;

    Ok(())
}
