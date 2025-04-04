use axum::routing::post;
use axum::Router;

use handlers::{
    authenticate_handler, config_handler, create_event_emitters_handler, init_handler,
    initialize_domain_services_handler, load_profile_handler, verify_public_key_in_agent_handler,
};

use crate::registry::StateRegistry;

pub mod handlers;
pub mod models;

pub fn router() -> Router<StateRegistry> {
    Router::new()
        .route("/load_profile", post(load_profile_handler))
        .route(
            "/initialize_domain_services",
            post(initialize_domain_services_handler),
        )
        .route(
            "/create_event_emitters",
            post(create_event_emitters_handler),
        )
        .route("/config", post(config_handler))
        .route("/authenticate", post(authenticate_handler))
        .route(
            "/verify_public_key_in_agent",
            post(verify_public_key_in_agent_handler),
        )
        .route("/init", post(init_handler))
}
