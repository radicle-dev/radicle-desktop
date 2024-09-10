use anyhow::anyhow;

use radicle::crypto::ssh;

use crate::{error::Error, AppState};

#[tauri::command]
pub fn authenticate(ctx: tauri::State<AppState>) -> Result<(), Error> {
    let profile = &ctx.profile;

    if !profile.keystore.is_encrypted()? {
        return Ok(());
    }
    match ssh::agent::Agent::connect() {
        Ok(mut agent) => {
            if agent.request_identities()?.contains(&profile.public_key) {
                Ok(())
            } else {
                Err(Error::WithHint {
                    err: anyhow!("Not able to find your keys in the ssh agent"),
                    hint: "Make sure to run <code>rad auth</code> in your terminal to add your keys to the ssh-agent.",
                })?
            }
        }
        Err(e) if e.is_not_running() => Err(Error::WithHint { err: anyhow!("SSH Agent is not running"), hint: "For now we require the user to have an ssh agent running, since we don't have passphrase inputs yet." })?, 
        Err(e) => Err(e)?,
    }
}
