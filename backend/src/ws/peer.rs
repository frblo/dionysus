use axum::extract::ws::WebSocket;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use yrs_axum::{
    broadcast::BroadcastGroup,
    ws::{AxumSink, AxumStream},
};

use crate::rooms::RoomManager;

pub async fn peer(ws: WebSocket, rooms: RoomManager, bcast: Arc<BroadcastGroup>, room_id: String) {
    let (sink, stream) = ws.split();
    let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
    let stream = AxumStream::from(stream);

    let sub = bcast.subscribe(sink, stream);
    match sub.completed().await {
        Ok(()) => println!("room={room_id} finished successfully"),
        Err(e) => eprintln!("room={room_id} finished abruptly: {e}"),
    }

    rooms.disconnect(&room_id).await;
}
