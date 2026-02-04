mod app;
mod auth;
mod db;
mod rooms;
mod state;
mod ws;

use std::net::SocketAddr;

use sqlx::PgPool;

use crate::db::Db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddr = "0.0.0.0:8000".parse().unwrap();
    let pool = PgPool::connect(&dotenvy::var("DATABASE_URL")?).await?;

    let state = state::AppState::new(Db::new(pool)).await;

    let app = app::router(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
