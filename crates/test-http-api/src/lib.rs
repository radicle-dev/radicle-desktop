use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

use radicle::Profile;

mod api;

#[derive(Debug, Clone)]
pub struct Options {
    pub listen: SocketAddr,
}

pub async fn run(options: Options) -> anyhow::Result<()> {
    let profile = Profile::load()?;
    let listener = TcpListener::bind(options.listen).await?;
    let app = router(profile)?.into_make_service_with_connect_info::<SocketAddr>();

    axum::serve(listener, app)
        .await
        .map_err(anyhow::Error::from)
}

fn router(profile: Profile) -> anyhow::Result<Router> {
    let profile = Arc::new(profile);
    let ctx = api::Context::new(profile);

    Ok(api::router(ctx))
}
