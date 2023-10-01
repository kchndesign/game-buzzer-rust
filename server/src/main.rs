use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::Message;
use warp::{Filter, filters::ws::WebSocket};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    // establish in memory state
    
    let buzzer_activated = RwLock::new(false);
    let user_activated = Option::from(None);
    let teams= RwLock::new(HashMap::new());

    let state: Arc<State> = Arc::new(State {
        buzzer_activated,
        user_activated,
        teams
    });


    // Lay out routes, middleware and actions for each route
    let routes = warp::path("echo")
        // ws() is a filter that will handle the handshake for 
        // requests to this path
        .and(warp::ws())
        // Init a new closure with the upgrade object that 
        // is spit out by the ws() filter
        .map(move | upgrade: warp::ws::Ws | {
            let cloned_state = state.clone();
            upgrade.on_upgrade(| websocket| on_upgrade(websocket, cloned_state))
        });

    warp::serve(routes).run(([127, 0, 0, 1], 5000)).await;
}

// handle new connections
async fn on_upgrade(websocket: WebSocket, state: Arc<State>) {
    let (mut ws_send, mut ws_receive) = websocket.split();

    // create channel for this user
    // then create receiver stream for the receiver side of the channel we just created.
    let (channel_send, channel_receive) = mpsc::unbounded_channel();
    let mut channel_receiver_stream = UnboundedReceiverStream::new(channel_receive);

    // spawn a new task that will handle any incoming channel messages and immediately send them to the websocket sender
    tokio::task::spawn(async move {
        while let Some(incoming_channel_message) = channel_receiver_stream.next().await {
            ws_send.send(incoming_channel_message).await
                .unwrap_or_else(| error | {
                    eprintln!("Could not send websocket message from channel: {}", error);
                });
        }
    });

    // explicitly handle the first message as a registration message 
    let first_message = ws_receive.next().await.unwrap();
    let first_message = first_message.unwrap();

    // all messages should be text
    assert!(first_message.is_text());

    // get the text for the message
    let text = first_message.to_str().unwrap();
    let sections: Vec<&str> = text.split(".").collect();

    if sections.len() != 5 {
        println!("Received message with wrong number of sections (should be 5) {}", text);
        return;
    }

    // the first section describes the type of the message
    match sections[0] {

        // register new user to team
        "register" => {
            let user = sections[1];
            let team = sections[2];
            // // wait for the lock to clear on the all users vector before inserting
            // state.all_users.write().await
            //     .insert(user.to_string(), &channel_send);
            
            let mut teams = state.teams.write().await;
            let maybe_team = teams.get_mut(team);

            match maybe_team {
                Some(value) => {
                    value.insert(user.to_string(), channel_send);
                },
                None => {
                    let mut new_team = HashMap::new();
                    new_team.insert(user.to_string(), channel_send);
                    state.teams.write().await.insert(team.to_string(), new_team);
                }
            }
        },
        &_ => {}
    };



    // while there are some websocket messages to receive from this user, handle them
    while let Some(message) = ws_receive.next().await {
        let msg = message.unwrap();

        // all messages should be text
        assert!(msg.is_text());

        // get the text for the message
        let text = msg.to_str().unwrap();
        let sections: Vec<&str> = text.split(".").collect();

        if sections.len() != 5 {
            println!("Received message with wrong number of sections (should be 5) {}", text);
            continue;
        }

        // the first section describes the type of the message
        match sections[0] {

            // register new user to team
            "register" => {
                
            },
            &_ => {}
        }

        // let _ = ws_send.send(message.unwrap()).await;
    }
}

pub struct State {
    pub buzzer_activated: RwLock<bool>,
    pub user_activated: Option<String>,
    pub teams: RwLock<HashMap<String, HashMap<String, mpsc::UnboundedSender<Message>>>>
}