use serde::{Deserialize, Serialize};

#[derive(Deserialize, PartialEq, Eq)]
pub enum InboundMessageType {
    Register,
    Buzzer,
    SelectQuestion,
    Answer
}

#[derive(Serialize)]
pub enum OutboundMessageType {
    NewUser,
    BuzzerReady,
    BuzzerActivated,
    DisconnectedUser
}

#[derive(Deserialize)]
pub struct IncomingMessage {
    pub message_type: InboundMessageType,
    pub team: Option<String>,
    pub user: Option<String>,
    pub body: Option<String>
}

#[derive(Serialize)]
pub struct OutgoingMessage {
    pub message_type: OutboundMessageType,
    pub team: String,
    pub user: String,
    pub body: String,
}

