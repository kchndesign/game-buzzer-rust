use std::collections::HashMap;
use std::sync::Arc;
use gamestate::MultiplayerActorSink;
use gamestate::MultiplayerMessage;
use player_interface::PlayerRegistrationMessage;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::Message;
use warp::{Filter, filters::ws::WebSocket};
use futures_util::{SinkExt, StreamExt};
mod gamestate;
mod player_interface;

#[tokio::main]
async fn main() {
    // in memory state
    let games: HashMap<String, Arc<MultiplayerActorSink>> = HashMap::new();

    // Lay out routes, middleware and actions for each route
    let ws_entry_player = warp::path!("game" / String)
        // ws() is a filter that will handle the handshake for 
        // requests to this path
        .and(warp::ws())
        // Init a new closure with the upgrade object that 
        // is spit out by the ws() filter
        .map(move | game_id: String, upgrade: warp::ws::Ws | {
            let game = games.get(&game_id).unwrap().clone();
            upgrade.on_upgrade(| websocket| on_upgrade_player(websocket, game))
        });

    let routes = ws_entry_player;

    warp::serve(routes).run(([127, 0, 0, 1], 5000)).await;
}

// handle new connections
async fn on_upgrade_player(websocket: WebSocket, state: Arc<MultiplayerActorSink> ) {
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

    let maybe_registration_message = PlayerRegistrationMessage::new(first_message_actual.to_str().unwrap());
    if maybe_registration_message.is_none() {
        eprintln!("Player registration message was invalid");
        return;
    }
    
    // store the user's registration information
    let registration_message = maybe_registration_message.unwrap();
    let team = registration_message.team;
    let user_name = registration_message.user_name;
    state.handle_message(MultiplayerMessage::RegisterUserForTeam { team: team.clone(), user_name: user_name.clone(), channel: channel_send }).await;

    // spawn a new task that will handle any future incoming messages for this client.
    tokio::task::spawn(async move {
        // while there are some websocket messages to receive from this user, handle them
        while let Some(message) = ws_receive.next().await {
            let msg = message.unwrap();

            // handle disconnect message
            if msg.is_close() {
                state.handle_message(MultiplayerMessage::DisconnectUser { team: team.clone(), user_name: user_name.clone()}).await;
                return;
            }

            // all non disconnect messages should be text
            // ignore any that aren't
            if !msg.is_text() {
                continue;
            }

            // // parse the message json
            // let json_result = serde_json::from_str::<IncomingMessage>(msg.to_str().unwrap());

            // if json_result.is_err() {
            //     eprintln!("Incoming message was not valid JSON: {}", msg.to_str().unwrap());
            //     eprintln!("Invalid message was received from: {}", user_name.clone());
            // }

            // let msg_body = json_result.unwrap();
            // match msg_body.message_type {
            //     InboundMessageType::Register => {
            //         eprintln!("Received another registration attempt when already registered");
            //     },

            //     InboundMessageType::Buzzer => {
            //     state.handle_message(MultiplayerMessage::ActivateBuzzer { team: team.clone(), user_name: user_name.clone()}).await;
            //     },

            //     _ => {
                    
            //     }
            // }
        }
    });
}

pub struct State {
    pub buzzer_activated: RwLock<bool>,
    pub user_activated: Option<String>,
    pub teams: RwLock<HashMap<String, HashMap<String, mpsc::UnboundedSender<Message>>>>
}