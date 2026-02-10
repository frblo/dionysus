mod app;
mod auth;
mod db;
mod rooms;
mod state;
mod ws;

use std::{env, net::SocketAddr};

use sqlx::PgPool;

use crate::db::Db;

const SERVE_FRONTEND_ARG: &str = "serve";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddr = "0.0.0.0:8000".parse().unwrap();
    let pool = PgPool::connect(&dotenvy::var("DATABASE_URL")?).await?;

    let state = state::AppState::new(Db::new(pool)).await;

    let args: Vec<String> = env::args().collect();

    let serve_frontend = match &args.get(1) {
        Some(s) => {
            if s.as_str() == SERVE_FRONTEND_ARG {
                true
            } else {
                false
            }
        }
        _ => false,
    };

    let app = app::router(state, serve_frontend);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
