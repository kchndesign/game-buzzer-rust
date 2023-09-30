use std::collections::HashMap;

use warp::{Filter, filters::ws::WebSocket};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    // Establish in-memory state
    let usernames = vec![""; 255];

    // Lay out routes, middleware and actions for each route
    let routes = warp::path("echo")
        // ws() is a filter that will handle the handshake for 
        // requests to this path
        .and(warp::ws())
        // Init a new closure with the upgrade object that 
        // is spit out by the ws() filter
        .map(| upgrade: warp::ws::Ws | {
            upgrade.on_upgrade(| websocket | on_upgrade(websocket))
        });

    warp::serve(routes).run(([127, 0, 0, 1], 5000)).await;
}

// handle new connections
async fn on_upgrade(websocket: WebSocket) {
    let (mut send, mut receive) = websocket.split();

    // while there are stream elements to send, 
    // send that element straight back to the sink
    while let Some(message) = receive.next().await {
        let _ = send.send(message.unwrap()).await;
    }
}