use axum::{extract::State, response::IntoResponse, Json};
use radicle::cob::cache::COBS_DB_FILE;
use radicle::node::NOTIFICATIONS_DB_FILE;
use radicle::Profile;

use radicle_types::config::Config;
use radicle_types::domain::identity::service::Service as IdentityService;
use radicle_types::domain::identity::traits::IdentityService as _;
use radicle_types::domain::inbox::service::Service as InboxService;
use radicle_types::domain::repo::service::Service as RepoService;
use radicle_types::error::Error;
use radicle_types::outbound::{radicle::Radicle, sqlite::Sqlite};

use crate::registry::StateRegistry;

use super::models::{AuthPayload, Init};

pub(crate) async fn load_profile_handler(
    State(app_state): State<StateRegistry>,
) -> impl IntoResponse {
    let profile = radicle::Profile::load()?;
    app_state.manage(profile.clone()).await;

    Ok::<_, Error>(Json(Config::get(&profile)))
}

pub(crate) async fn create_event_emitters_handler() -> impl IntoResponse {
    Ok::<_, Error>(Json(()))
}

pub(crate) async fn create_services_handler(
    State(app_state): State<StateRegistry>,
) -> impl IntoResponse {
    let profile = app_state.state::<Profile>().await.unwrap();
    let inbox_db = Sqlite::reader(profile.node().join(NOTIFICATIONS_DB_FILE))?;
    let cob_db = Sqlite::reader(profile.cobs().join(COBS_DB_FILE))?;
    let radicle = Radicle::new((*profile).clone());

    let inbox_service = InboxService::new(inbox_db);
    let repo_service = RepoService::new(radicle.clone(), cob_db);
    let identity_service = IdentityService::new(radicle);

    app_state.manage(inbox_service).await;
    app_state.manage(repo_service).await;
    app_state.manage(identity_service).await;

    Ok::<_, Error>(Json(()))
}

pub(crate) async fn config_handler(State(app_state): State<StateRegistry>) -> impl IntoResponse {
    let profile = app_state.state::<Profile>().await.unwrap();

    Ok::<_, Error>(Json(Config::get(&profile)))
}

pub(crate) async fn authenticate_handler(
    State(app_state): State<StateRegistry>,
    Json(payload): Json<AuthPayload>,
) -> impl IntoResponse {
    let service = app_state.state::<IdentityService<Radicle>>().await.unwrap();
    service.authenticate(payload.passphrase.into())?;

    Ok::<_, Error>(Json(()))
}

pub(crate) async fn check_agent_handler(
    State(app_state): State<StateRegistry>,
) -> impl IntoResponse {
    let profile = app_state.state::<Profile>().await.unwrap();
    match radicle::crypto::ssh::agent::Agent::connect() {
        Ok(mut agent) => {
            if agent.request_identities()?.contains(&profile.public_key) {
                Ok(Json(()))
            } else {
                Err(Error::AgentNotRunning)
            }
        }
        Err(e) if e.is_not_running() => Err(Error::AgentNotRunning)?,
        Err(e) => Err(e)?,
    }
}

pub(crate) async fn init_handler(
    Json(Init { alias, passphrase }): Json<Init>,
) -> impl IntoResponse {
    Radicle::init(alias, passphrase.into())?;

    Ok::<_, Error>(Json(()))
}
