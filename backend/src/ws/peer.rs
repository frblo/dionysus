use axum::extract::ws::WebSocket;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use yrs_axum::ws::{AxumSink, AxumStream};

use crate::state::AppState;

pub async fn peer(ws: WebSocket, state: AppState, room_id: String) {
    let bcast = state.ensure_room(&room_id).await;

    let (sink, stream) = ws.split();
    let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
    let stream = AxumStream::from(stream);

    let sub = bcast.subscribe(sink, stream);
    match sub.completed().await {
        Ok(_) => println!("room={} finished successfully", room_id),
        Err(e) => eprintln!("room={} finished abruptly: {}", room_id, e),
    }
}
