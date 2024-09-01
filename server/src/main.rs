use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use admin_interface::AdminRegistration;
use gamestate::GameState;
use gamestate::MultiplayerActorSink;
use gamestate::MultiplayerGameManager;
use gamestate::MultiplayerMessage;
use player_interface::PlayerRegistrationMessage;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::Message;
use warp::filters::ws::Ws;
use warp::{Filter, filters::ws::WebSocket};
use futures_util::{SinkExt, StreamExt};
mod gamestate;
mod player_interface;
mod admin_interface;

#[tokio::main]
async fn main() {
    // in memory state
    let games: Arc<RwLock<HashMap<String, Arc<MultiplayerActorSink>>>> = Arc::new(RwLock::new(HashMap::new()));
    let games_filter = warp::any().map(move || games.clone());

    let ws_entry_admin = warp::path("create")
        .and(warp::query::<HashMap<String, String>>())
        .and(games_filter.clone())
        .and(warp::ws())
        .and_then(| query: HashMap<String, String>,  games: Arc<RwLock<HashMap<String, Arc<MultiplayerActorSink>>>>, upgrade: Ws | async move {
            if !query.contains_key("teams") {
                return Err(warp::reject());
            }

            let body = AdminRegistration {
                teams: serde_json::from_str(query.get("teams").unwrap().as_str()).unwrap()
            };

            let mut games_dict = games.write().await;
            let mut game_id: String = "debug".to_string();
            loop {
                if !games_dict.contains_key(&game_id) { 
                    break;
                }

                game_id = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(5)
                    .map(char::from)
                    .collect::<String>();
            }
            let mut teams = HashMap::new();
            for team in body.teams {
                teams.insert(team, Vec::new());
            }

            let new_game_actor = MultiplayerGameManager::new_actor(GameState::new(teams));
            let arc_game_actor = Arc::from(new_game_actor);
            let arc_game_actor_for_channels = arc_game_actor.clone();
            games_dict.insert(game_id.clone(), arc_game_actor);
            Ok(upgrade.on_upgrade(move| websocket| on_upgrade_admin(websocket, arc_game_actor_for_channels, game_id)))
        });

    // Lay out routes, middleware and actions for each route
    let ws_entry_player = warp::path!("game" / String)
    // ws() is a filter that will handle the handshake for 
    // requests to this path
        .and(games_filter.clone())
        .and(warp::ws())
        // Init a new closure with the upgrade object that 
        // is spit out by the ws() filter
        .and_then(| game_id: String, games: Arc<RwLock<HashMap<String, Arc<MultiplayerActorSink>>>>, upgrade: Ws | async move {
            let game = games.read().await.get(&game_id).unwrap().clone();
            Ok::<_, Infallible>(upgrade.on_upgrade(| websocket| on_upgrade_player(websocket, game)))
        });

    let routes = ws_entry_player.or(ws_entry_admin);

    warp::serve(routes).run(([127, 0, 0, 1], 5000)).await;
}

async fn on_upgrade_admin(websocket: WebSocket, state: Arc<MultiplayerActorSink>, game_id: String) {
    let (mut ws_send, mut ws_receive) = websocket.split();

    // create channel for admin
    // then create receiver stream for the receiver side of the channel we just created.
    let (channel_send, channel_receive) = mpsc::unbounded_channel();
    let mut channel_receiver_stream = UnboundedReceiverStream::new(channel_receive);

    // spawn a new task that will listen to the channel and relay the messages to the websocket sender
    tokio::task::spawn(async move {
        while let Some(incoming_channel_message) = channel_receiver_stream.next().await {
            let ws_send_result = ws_send.send(incoming_channel_message).await;
            match ws_send_result {
                Err(error) => {
                    eprintln!("Could not send for websocket channel: {}, closing ws channel", error);
                    let _ = ws_send.close().await;
                    channel_receiver_stream.close();
                    break;
                },
                Ok(_) => {}
            }
        }
    });

    // explicitly handle first message to admin, which is to tell the admin the game ID
    channel_send.send(Message::text(game_id)).unwrap_or_else(|err| {
        eprintln!("Could not send game ID message to admin {}", err);
    });

    // add the admin as a user in the game 
    state.handle_message(MultiplayerMessage::AddAdminUser { channel: channel_send }).await;

    // spawn task for ongoing admin messages
    tokio::task::spawn(async move {
        while let Some(message) = ws_receive.next().await {
            let msg = message.unwrap();

            // handle disconnect message
            if msg.is_close() {
                state.handle_message(MultiplayerMessage::DiscardGame {  }).await;
                return;
            }

            // all non disconnect messages should be text
            // ignore any that aren't
            if !msg.is_text() {
                continue;
            }

            let message_text = msg.to_str().unwrap();
            match message_text {
                "reset_buzzer" => state.handle_message(MultiplayerMessage::ResetBuzzer {  }).await,
                something_else => eprintln!("Received unrecognised input from admin: {}", something_else)
            }
        }
    });
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
            let ws_send_result = ws_send.send(incoming_channel_message).await;
            match ws_send_result {
                Err(error) => {
                    eprintln!("Could not send for websocket channel: {}, closing ws channel", error);
                    let _ = ws_send.close().await;
                    channel_receiver_stream.close();
                    break;
                },
                Ok(_) => {}
            }
        }
    });

    {
        // instantly send a message to the player on upgrade to tell the player which teams are available to choose.
        let (get_teams_sink, get_teams_listener) = oneshot::channel::<Vec<String>>();
        state.handle_message(MultiplayerMessage::GetTeams { respond_to: get_teams_sink }).await;
        let result = get_teams_listener.await.unwrap();
        let _ = channel_send.send(Message::text(serde_json::to_string(&result).unwrap()));
    }
    
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

            let msg_text = msg.to_str().unwrap();
            match msg_text {
                "activate_buzzer" => state.handle_message(MultiplayerMessage::ActivateBuzzer { team: team.clone(), user_name: user_name.clone() }).await,
                something_else => eprintln!("Received unrecognised input from player: {}", something_else)
            }
        }
    });
}

pub struct State {
    pub buzzer_activated: RwLock<bool>,
    pub user_activated: Option<String>,
    pub teams: RwLock<HashMap<String, HashMap<String, mpsc::UnboundedSender<Message>>>>
}