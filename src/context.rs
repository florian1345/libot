use std::ops::Deref;
use crate::model::game::{Color, GameInfo};
use crate::model::user::UserId;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BotContext {

    /// The [UserId] of this bot's user.
    pub bot_id: UserId
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GameContext {

    /// The [UserId] of this bot's user.
    pub bot_id: UserId,

    /// The [Color] as which this bot plays, or [None] if it is not a participant.
    pub bot_color: Option<Color>,

    pub(crate) info: GameInfo
}

impl Deref for GameContext {

    type Target = GameInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}
