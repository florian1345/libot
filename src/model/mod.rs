use serde::Deserialize;

use crate::model::game::Clock;
use crate::model::user::UserId;

pub mod user;
pub mod game;
pub mod challenge;
pub mod bot_event;
pub(crate) mod request;

pub type Move = String;
pub type Moves = String;
pub type Milliseconds = i64;
pub type Seconds = i32;
pub type Days = i32;
pub type Timestamp = i64;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct Compat {
    // TODO Option<bool> correct?
    pub bot: Option<bool>,
    pub board: Option<bool>
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TimeControl {
    Clock(Clock),
    #[serde(rename_all = "camelCase")]
    Correspondence {
        // TODO really optional?
        days_per_turn: Option<Days>
    },
    Unlimited
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct UserProfile {
    pub id: UserId,
    pub username: String
}
