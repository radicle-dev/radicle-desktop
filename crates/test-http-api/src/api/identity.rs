use axum::{routing::post, Router};
use handlers::{
    authenticate_handler, check_agent_handler, config_handler, create_event_emitters_handler,
    create_services_handler, init_handler, load_profile_handler,
};

use crate::registry::StateRegistry;

pub mod handlers;
pub mod models;

pub fn router() -> Router<StateRegistry> {
    Router::new()
        .route("/load_profile", post(load_profile_handler))
        .route("/create_services", post(create_services_handler))
        .route(
            "/create_event_emitters",
            post(create_event_emitters_handler),
        )
        .route("/config", post(config_handler))
        .route("/authenticate", post(authenticate_handler))
        .route("/check_agent", post(check_agent_handler))
        .route("/init", post(init_handler))
}
