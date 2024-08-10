use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AdminRegistration {
    pub teams: Vec<String>
}