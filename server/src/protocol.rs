use serde::{Deserialize, Serialize};

pub enum InboundMessageType {
    Register,
    Buzzer,
    SelectQuestion,
    Answer
}

impl InboundMessageType {
    pub fn from(input: &str) -> InboundMessageType {
        match input {
            "register" => InboundMessageType::Register,
            "buzzer" => InboundMessageType::Buzzer,
            "select-question" => InboundMessageType::SelectQuestion,
            "answer" => InboundMessageType::Answer,
            &_ => InboundMessageType::Buzzer
        }
    }
}

#[derive(Serialize)]
pub enum OutboundMessageType {
    NewUser,
    BuzzerReady,
    BuzzerActivated
}

impl OutboundMessageType {
    pub fn from(input: &str) -> OutboundMessageType {
        match input {
            "new-user" => OutboundMessageType::NewUser,
            "buzzer-ready" => OutboundMessageType::BuzzerReady,
            "buzzer_activated" => OutboundMessageType::BuzzerActivated,
            &_ => OutboundMessageType::BuzzerActivated
        }
    }
}

pub struct IncomingMessage {
    pub message_type: InboundMessageType,
    pub team: String,
    pub user: String,
    pub body: String,
}

#[derive(Serialize)]
pub struct OutgoingMessage {
    pub message_type: OutboundMessageType,
    pub team: String,
    pub user: String,
    pub body: String,
}

