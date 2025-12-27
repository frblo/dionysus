use axum::extract::ws::WebSocket;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use yrs_axum::{
    broadcast::BroadcastGroup,
    ws::{AxumSink, AxumStream},
};

pub async fn peer(ws: WebSocket, bcast: Arc<BroadcastGroup>, room_id: String) {
    let (sink, stream) = ws.split();
    let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
    let stream = AxumStream::from(stream);

    let sub = bcast.subscribe(sink, stream);
    match sub.completed().await {
        Ok(_) => println!("room={} finished successfully", room_id),
        Err(e) => eprintln!("room={} finished abruptly: {}", room_id, e),
    }
}
