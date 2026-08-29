use axum::extract::ws::WebSocket;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use yrs::sync::{DefaultProtocol, Protocol};
use yrs::updates::encoder::{Encoder, EncoderV1};
use yrs_axum::{
    broadcast::BroadcastGroup,
    ws::{AxumSink, AxumStream},
};

use crate::rooms::RoomManager;

/// Drives a single client's websocket connection to `room_id` for its whole
/// lifetime. Sends the server's initial sync message, hands the socket to the
/// room's [`BroadcastGroup`] to run the Yjs sync protocol until the connection
/// ends, then releases it via [`RoomManager::disconnect`].
///
/// `request_id` is passed in from `ws_handler` rather than read from a span:
/// this runs in a task detached from the request by `on_upgrade`, so it can't
/// inherit the `http_request` span the way the rest of the handler does.
#[tracing::instrument(skip_all, fields(room_id = %room_id, request_id = %request_id))]
pub async fn peer(
    ws: WebSocket,
    rooms: RoomManager,
    bcast: Arc<BroadcastGroup>,
    room_id: String,
    request_id: String,
) {
    let (sink, stream) = ws.split();
    let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
    let stream = AxumStream::from(stream);

    // Workaround until we replace yrs-axum:
    // yrs-axum's BroadcastGroup::subscribe only ever replies to a client-sent
    // SyncStep1 and it never sends the server's own SyncStep1. Without that,
    // the server can push updates to a client but never pull updates back
    // from one (e.g. edits a client made while its socket was disconnected),
    // so send it here explicitly before handing the connection off.
    let init_payload = {
        let awareness = bcast.awareness().read().await;
        let mut encoder = EncoderV1::new();
        match DefaultProtocol.start(&awareness, &mut encoder) {
            Ok(()) => Some(encoder.to_vec()),
            Err(e) => {
                tracing::warn!(error = %e, "failed to build initial sync message");
                None
            }
        }
    };
    if let Some(payload) = init_payload
        && !payload.is_empty()
    {
        let mut s = sink.lock().await;
        if let Err(e) = s.send(payload).await {
            tracing::warn!(error = %e, "failed to send initial sync message");
        }
    }

    let sub = bcast.subscribe(sink, stream);
    match sub.completed().await {
        Ok(()) => tracing::info!("websocket connection closed"),
        Err(e) => tracing::warn!(error = %e, "websocket connection closed abnormally"),
    }

    rooms.disconnect(&room_id).await;
}
