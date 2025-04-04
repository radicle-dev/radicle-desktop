use std::ops::Deref as _;

use radicle::crypto::ssh::Passphrase;
use tauri::{AppHandle, Manager};

use radicle_types::domain::identity::service::Service;
use radicle_types::domain::identity::traits::IdentityService as _;
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle;

#[tauri::command]
pub fn authenticate(
    service: tauri::State<Service<Radicle>>,
    passphrase: Passphrase,
) -> Result<(), Error> {
    service.authenticate(passphrase)
}

#[tauri::command]
pub fn verify_public_key_in_agent(profile: tauri::State<radicle::Profile>) -> Result<(), Error> {
    Service::<Radicle>::verify_public_key_in_agent(profile.public_key)
}

#[tauri::command]
pub fn init(app: AppHandle, alias: String, passphrase: Passphrase) -> Result<(), Error> {
    let radicle = Radicle::init(alias, passphrase)?;
    app.manage(radicle.profile().deref().clone());

    Ok(())
}
