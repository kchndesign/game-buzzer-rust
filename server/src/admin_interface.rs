use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AdminPostBody {
    pub teams: Vec<String>
}