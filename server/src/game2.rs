use std::{collections::HashMap, ops::Deref, sync::Arc};
use futures_util::StreamExt;
use serde::Serialize;
use tokio::sync::{mpsc::{self, UnboundedSender}, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::Message;

#[derive(Serialize)]
pub struct GameState {
    pub buzzer_activated: bool,
    pub user_activated: Option<String>,
    pub team_activated: Option<String>,
    pub teams: HashMap<String, Vec<String>>
}

impl GameState {
    pub fn new(teams: HashMap<String, Vec<String>>) -> GameState {
        return Self {
            buzzer_activated: false,
            user_activated: Option::None,
            team_activated: Option::None,
            teams
        }
    }
}

pub enum MultiplayerMessage {
    RegisterUserForTeam {
        team: String,
        user_name: String,
        channel: UnboundedSender<Message>
    },
    DisconnectUser {
        team: String,
        user_name: String,
    },
    ActivateBuzzer {
        team: String,
        user_name: String,
    },
    DiscardGame {}
}

pub struct MultiplayerActorSink {
    sender: UnboundedSender<MultiplayerMessage>
}

impl MultiplayerActorSink {
    pub fn new(sender: UnboundedSender<MultiplayerMessage>) -> Self {
        return Self {
            sender
        }
    }

    pub async fn handle_message(&self, message: MultiplayerMessage) {
        self.sender.send(message).unwrap_or_else(|error| 
            eprintln!("Could not handle message for multiplayer game: {}", error));
    }
}

pub struct MultiplayerGameManager {
    state: Arc<RwLock<GameState>>,
    all_users: RwLock<Vec<mpsc::UnboundedSender<Message>>>
}

impl MultiplayerGameManager {

    /// Implements an actor pattern over the game manager and returns the channel that can be used to make mutations to the game.
    pub fn new_actor(state: GameState) -> MultiplayerActorSink {
        let (tx, rx) = mpsc::unbounded_channel::<MultiplayerMessage>();
        let mut rx_stream = UnboundedReceiverStream::new(rx);
        let mut game_manager = Self::new(state);

        tokio::spawn(async move {
            while let Some(actor_message) = rx_stream.next().await {
                match actor_message {
                    MultiplayerMessage::RegisterUserForTeam { team, user_name, channel} => {
                        game_manager.register_user_for_team(team, user_name, channel).await;
                    }
                    MultiplayerMessage::DisconnectUser { team, user_name } => 
                        game_manager.disconnect_user(team, user_name).await,
                    MultiplayerMessage::ActivateBuzzer { team, user_name } => 
                        game_manager.try_activate_buzzer(team, user_name).await,
                    MultiplayerMessage::DiscardGame {} => 
                        break
                }
            }
        });

        return MultiplayerActorSink::new(tx);
    }

    pub fn new(state: GameState) -> MultiplayerGameManager {
        return Self {
            state: Arc::from(RwLock::new(state)),
            all_users: RwLock::new(Vec::new()),
        }
    }

    /// Use post mutation to the state object to send the state to all users.
    async fn update_state_for_all_users(&self) {
        let state = self.state.read().await;
        let message_text = serde_json::to_string(state.deref()).unwrap();

        // TODO: this is probably a bad idea
        self.all_users.write().await.retain(|channel| {
            let send_result = channel.send(Message::text(&message_text));
            send_result.is_ok()
        });
    }

    /// Handle new user
    /// returns true when registration is successful, else returns false (when team does not exist for example).
    pub async fn register_user_for_team(&mut self, team: String, user_name: String, channel: mpsc::UnboundedSender<Message>) -> bool {
        // add user to the appropriate team
        {
            let mut state = self.state.write().await;
            let maybe_team = state.teams.get_mut(&team);

            match maybe_team {
                Some(value) => {
                    println!("Found existing team to add for user {}", user_name);
                    value.push(user_name.to_string());
                },
                None => {
                    println!("Could not find {} team for user {}", team, user_name);
                    return false;
                }
            }
        }

        // add user's WS channel to the list of all_users
        self.all_users.write().await.push(channel);

        // broadcast update
        self.update_state_for_all_users().await;

        return true;
    }

    pub async fn disconnect_user (&mut self, team: String, user_name: String) {
        // remove user from game state
        {
            let mut state = self.state.write().await;
            let maybe_team = state.teams.get_mut(&team);

            match maybe_team {
                Some(value) => { 
                    value.retain(|x| *x == user_name);
                },
                None => {
                    println!("Could not find {} team for user {}", team, user_name);
                }
            }
        }

        // remove user from list of channels
        // TODO: it's probably a bad idea, but we'll just remove the channel next time we try to send something to it and it errors

        // broadcast update
        self.update_state_for_all_users().await;
    }

    pub async fn try_activate_buzzer(&mut self, team: String, user_name: String) {
        let mut state = self.state.write().await;
        let maybe_team = state.teams.get(&team);

        match maybe_team {
            Some(value) => { 
                if !value.iter().any(|x| *x == user_name) {
                    println!("Player {} does not exist in team {}", user_name, team);
                    return;
                }

                if state.buzzer_activated {
                    println!("Player {} tried to activate buzzer but it was already activated", user_name);
                    return;
                }

                state.buzzer_activated = true;
                state.team_activated = Some(team.to_string());
                state.user_activated = Some(user_name.to_string());
            },
            None => {
                println!("Could not find {} team for user {}", team, user_name);
            }
        }
    }

    pub async fn reset_buzzer(&mut self) {
        let mut state = self.state.write().await;
        state.buzzer_activated = false;
        state.team_activated = None;
        state.user_activated = None;
    }
}

