use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ChatRoom {
    Player,
    Spectator
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ChatLine {
    pub username: String,
    pub text: String
}

pub type ChatHistory = Vec<ChatLine>;
