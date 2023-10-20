use std::collections::HashMap;
use std::sync::Arc;
use protocol::InboundMessageType;
use protocol::IncomingMessage;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::Message;
use warp::{Filter, filters::ws::WebSocket};
use futures_util::{SinkExt, StreamExt};
mod game;
use crate::game::Game;
mod protocol;

#[tokio::main]
async fn main() {
    // establish in memory state
    let game = Arc::new(RwLock::new(Game::new()));

    // Lay out routes, middleware and actions for each route
    let routes = warp::path("echo")
        // ws() is a filter that will handle the handshake for 
        // requests to this path
        .and(warp::ws())
        // Init a new closure with the upgrade object that 
        // is spit out by the ws() filter
        .map(move | upgrade: warp::ws::Ws | {
            let cloned_state = game.clone();
            upgrade.on_upgrade(| websocket| on_upgrade(websocket, cloned_state))
        });

    warp::serve(routes).run(([127, 0, 0, 1], 5000)).await;
}

// handle new connections
async fn on_upgrade(websocket: WebSocket, state: Arc<RwLock<Game>>) {
    let (mut ws_send, mut ws_receive) = websocket.split();

    // create channel for this user
    // then create receiver stream for the receiver side of the channel we just created.
    let (channel_send, channel_receive) = mpsc::unbounded_channel();
    let mut channel_receiver_stream = UnboundedReceiverStream::new(channel_receive);

    // spawn a new task that will listen to the channel and relay the messages to the websocket sender
    tokio::task::spawn(async move {
        while let Some(incoming_channel_message) = channel_receiver_stream.next().await {
            ws_send.send(incoming_channel_message).await
                .unwrap_or_else(| error | {
                    eprintln!("Could not send websocket message from channel: {}", error);
                });
        }
    });
    
    // explicitly handle the first message (which should be a registration message)
    let first_message = ws_receive.next().await.unwrap();
    let first_message_actual = first_message.unwrap();
    if first_message_actual.is_close() {
        return;
    }

    let json_result = serde_json::from_str::<IncomingMessage>(first_message_actual.to_str().unwrap());
    if json_result.is_err() {
        eprintln!("Incoming message was not valid JSON: {}", first_message_actual.to_str().unwrap());
        return;
    }
    
    let first_message_body = json_result.unwrap();

    // assert that first message must be of type Register
    if first_message_body.message_type != InboundMessageType::Register {
        eprintln!("First message received was not a registration message");
        return;
    }

    // assert that the registration message must contain the team and user
    if first_message_body.team.is_none() || first_message_body.user.is_none() {
        eprintln!("Received registration message that was missing information");
        return;
    }
    
    // store the user's registration information
    let team_unwrap = first_message_body.team.unwrap();
    let user_unwrap = first_message_body.user.unwrap();
    state.write().await.register_user_for_team(&team_unwrap.as_str(), &user_unwrap.as_str(), channel_send).await;
    
    // spawn a new task that will handle any future incoming messages for this client.
    tokio::task::spawn(async move {
        // this is here to force tokio to persist first_message_body as part of the thread's closure
        let team = team_unwrap.as_str();
        let user = user_unwrap.as_str();

        // while there are some websocket messages to receive from this user, handle them
        while let Some(message) = ws_receive.next().await {
            let msg = message.unwrap();

            // handle disconnect message
            if msg.is_close() {
                state.write().await.disconnect_user(team, user).await;
                return;
            }

            // all non disconnect messages should be text
            // ignore any that aren't
            if !msg.is_text() {
                continue;
            }

            // parse the message json
            let json_result = serde_json::from_str::<IncomingMessage>(msg.to_str().unwrap());

            if json_result.is_err() {
                eprintln!("Incoming message was not valid JSON: {}", msg.to_str().unwrap());
                eprintln!("Invalid message was received from: {}", user);
            }

            let msg_body = json_result.unwrap();
            match msg_body.message_type {
                InboundMessageType::Register => {
                    eprintln!("Received another registration attempt when already registered");
                },

                InboundMessageType::Buzzer => {
                    state.write().await.try_activate_buzzer(team, user).await;
                },

                _ => {
                    
                }
            }
        }
    });
}

pub struct State {
    pub buzzer_activated: RwLock<bool>,
    pub user_activated: Option<String>,
    pub teams: RwLock<HashMap<String, HashMap<String, mpsc::UnboundedSender<Message>>>>
}