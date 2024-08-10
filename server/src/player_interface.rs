pub enum PlayerMessage { 
    Buzzer,
}

pub struct PlayerRegistrationMessage {
    pub team: String,
    pub user_name: String,
}

impl PlayerRegistrationMessage {
    pub fn new(message: &str) -> Option<Self> {
        let sections = message.split("|").collect::<Vec<&str>>();
        
        // validate that the sections exist:
        if sections.len() != 2 {
            return None;
        }

        // validate that they are all more than 2 letters
        let team = sections[0];
        let user_name = sections[1];

        if team.len() < 1 || user_name.len() < 1 {
            return None;
        }

        return Option::from(Self {
            team: team.to_string(),
            user_name: user_name.to_string()
        });
    }


}