mod app;
mod rooms;
mod state;
mod ws;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "0.0.0.0:8000".parse().unwrap();

    let state = state::AppState::new().await;

    let app = app::router(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
