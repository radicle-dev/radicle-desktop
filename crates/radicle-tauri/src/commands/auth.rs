use radicle::{crypto::ssh::Passphrase, Profile};

use radicle_types::domain::identity::{service::Service, traits::IdentityService};
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn authenticate(
    service: tauri::State<Service<Radicle>>,
    passphrase: Passphrase,
) -> Result<(), Error> {
    service.authenticate(passphrase)
}

#[tauri::command]
pub fn check_agent(profile: tauri::State<Profile>) -> Result<(), Error> {
    Service::<Radicle>::check_agent(profile.public_key)
}

#[tauri::command]
pub fn init(app: AppHandle, alias: String, passphrase: Passphrase) -> Result<(), Error> {
    let radicle = Radicle::init(alias, passphrase)?;
    app.manage((*radicle.profile()).clone());

    Ok(())
}
