mod app;
mod auth;
mod config;
mod db;
mod logging;
mod rooms;
mod state;
mod ws;

use std::net::SocketAddr;

use sqlx::PgPool;

use crate::{auth::AuthManager, db::Db};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Config::new()?;

    logging::init_tracing(&config)?;

    std::panic::set_hook(Box::new(|info| {
        tracing::error!(panic = %info, "panic occurred");
    }));

    let addr: SocketAddr = SocketAddr::new(config.listener.ip, config.listener.port);

    let pool = PgPool::connect(&config.database.url).await?;
    tracing::info!("connected to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to apply database migrations");
    tracing::info!("database migrations applied");

    let auth = AuthManager::new(&config).await?;

    let state = state::AppState::new(Db::new(pool), auth).await;

    let app = app::router(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!(addr = %listener.local_addr().unwrap(), "listening for connections");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
