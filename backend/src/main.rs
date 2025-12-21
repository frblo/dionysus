use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
    routing::get, Router
};
use yrs::sync::Awareness;
use yrs::{Doc, Text, Transact};
use yrs_axum::broadcast::BroadcastGroup;
use yrs_axum::ws::{AxumSink, AxumStream};
use yrs_axum::AwarenessRef;

#[tokio::main]
async fn main() {
    let awareness: AwarenessRef = {
        let doc = Doc::new();
        {
            let txt = doc.get_or_insert_text("codemirror");
            let mut txn = doc.transact_mut();
            txt.push(
                &mut txn,
                r#"EXT. BRICK'S PATIO - DAY

A gorgeous day.  The sun is shining.  But BRICK BRADDOCK, retired police detective, is sitting quietly, contemplating -- something.

The SCREEN DOOR slides open and DICK STEEL, his former partner and fellow retiree, emerges with two cold beers.

STEEL
Beer's ready!

BRICK
Are they cold?"#,
            );
        }
        Arc::new(RwLock::new(Awareness::new(doc)))
    };

    let bcast = Arc::new(BroadcastGroup::new(awareness.clone(), 32).await);

    let app = Router::new()
        .route("/demo-room-1", get(ws_handler))
        .with_state(bcast);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("Listening on {}", &listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(bcast): State<Arc<BroadcastGroup>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| peer(socket, bcast))
}

async fn peer(ws: WebSocket, bcast: Arc<BroadcastGroup>) {
    let (sink, stream) = ws.split();
    let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
    let stream = AxumStream::from(stream);
    let sub = bcast.subscribe(sink, stream);
    match sub.completed().await {
        Ok(_) => println!("broadcasting for channel finished successfully"),
        Err(e) => eprintln!("broadcasting for channel finished abruptly: {}", e),
    }
}
