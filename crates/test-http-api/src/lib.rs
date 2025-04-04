pub mod api;
pub mod registry;

use std::net::SocketAddr;

use axum::Router;
use hyper::{header::CONTENT_TYPE, Method};
use tokio::net::TcpListener;
use tower_http::cors;

use api::{identity, inbox, issue, patch, repo, thread};
use registry::StateRegistry;

#[derive(Debug, Clone)]
pub struct Options {
    pub listen: SocketAddr,
}

pub async fn run(options: Options) -> anyhow::Result<()> {
    let app_state = StateRegistry::default();

    let listener = TcpListener::bind(options.listen).await?;
    let app = Router::<StateRegistry>::new()
        .merge(identity::router())
        .merge(inbox::router())
        .merge(issue::router())
        .merge(patch::router())
        .merge(repo::router())
        .merge(thread::router())
        .layer(
            cors::CorsLayer::new()
                .allow_origin(cors::Any)
                .allow_methods([Method::POST])
                .allow_headers([CONTENT_TYPE]),
        )
        .with_state(app_state.clone());

    axum::serve(listener, app.into_make_service())
        .await
        .map_err(anyhow::Error::from)
}
