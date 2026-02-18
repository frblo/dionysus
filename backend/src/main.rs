mod app;
mod auth;
mod config;
mod db;
mod rooms;
mod state;
mod ws;

use std::net::SocketAddr;

use sqlx::PgPool;

use crate::db::Db;

const SERVE_FRONTEND_ARG: &str = "serve";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Config::new()?;

    let addr: SocketAddr = SocketAddr::new(config.listener.ip, config.listener.port);
    let pool = PgPool::connect(&config.database.url).await?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to apply database migrations");

    let state = state::AppState::new(Db::new(pool)).await;

    let app = app::router(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
