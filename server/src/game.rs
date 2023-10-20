use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use warp::filters::ws::Message;
use crate::protocol::{OutboundMessageType, OutgoingMessage};


pub struct Game {
    buzzer_activated: Mutex<bool>,
    user_activated: Option<String>,
    team_activated: Option<String>,
    teams: RwLock<HashMap<String, HashMap<String, Arc<mpsc::UnboundedSender<Message>>>>>,
    users: RwLock<Vec<Arc<mpsc::UnboundedSender<Message>>>>
}

impl Game {
    pub fn new() -> Game {
        return Self {
            buzzer_activated: Mutex::new(false),
            user_activated: Option::from(None),
            team_activated: Option::from(None),
            teams: RwLock::new(HashMap::new()),
            users: RwLock::new(Vec::new()),
        }
    }

    pub async fn register_user_for_team(&mut self, team: &str, user: &str, channel: mpsc::UnboundedSender<Message>) {
        // add the user to the appropriate team
        {
            let mut teams = self.teams.write().await;
            let maybe_team = teams.get_mut(team);
            let channel_reference = Arc::from(channel);
            self.users.write().await.push(channel_reference.clone());

            match maybe_team {
                Some(value) => {
                    println!("Found existing team to add for user {}", user);
                    value.insert(user.to_string(), channel_reference);
                },
                None =>{
                    println!("Creating new team for user {}", user);
                    let mut new_team = HashMap::new();
                    new_team.insert(user.to_string(), channel_reference);
                    teams.insert(team.to_string(), new_team);
                }
            }
        }

        // send a message to all users about the new user
        let message_data = OutgoingMessage {
            message_type: OutboundMessageType::NewUser,
            team: team.to_string(),
            user: user.to_string(),
            body: String::new()
        };

        let message = Message::text(serde_json::to_string(&message_data).unwrap());
        self.message_all_users(message).await;
    }

    pub async fn try_activate_buzzer(&mut self, team: &str, user: &str) {
        let is_locked = self.buzzer_activated.try_lock();

        match is_locked {
            Ok(_) => {
                println!("User {} successfully locked the thingy", user);
                self.user_activated = Option::from(user.to_string());
                self.team_activated = Option::from(team.to_string());
            },
            Err(_) => {
                println!("User {} failed to lock the buzzer, was probably already locked", user);
            }
        }
    }

    async fn message_all_users(&self, message: Message) {
        for channel in self.users.read().await.iter() {
            channel.send(message.clone()).unwrap();
        }
    }
}