use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeStatus {
    Created,
    Offline,
    Canceled,
    Declined,
    Accepted
}

// TODO replace with Option<Player>?
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeColor {
    White,
    Black,
    Random
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct ChallengePerf {
    pub icon: Option<String>,
    pub name: Option<String>
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeDirection {
    In,
    Out
}

/// An enumeration of the different reasons a bot can give why it rejected a challenge. This is
/// displayed to the challenger so they can potentially formulate a more conforming challenge.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeclineReason {

    /// Indicates that the bot does not accept challenges.
    Generic,

    /// Indicates that the bot does not accept challenges right now, but may later.
    Later,

    /// Indicates that the time control is too fast for the bot.
    TooFast,

    /// Indicates that the time control is too slow for the bot.
    TooSlow,

    /// Indicates that the bot does not accept challenges with the given time control.
    TimeControl,

    /// Indicates that the bot wants a rated challenge.
    Rated,

    /// Indicates that the bot wants a casual challenge.
    Casual,

    /// Indicates that the bot only accepts standard Chess.
    Standard,

    /// Indicates that the bot does not accept challenges of the given variant.
    Variant,

    /// Indicates that the bot does not accepts challenges from other bots.
    NoBot,

    /// Indicates that the bot only accepts challenges from other bots.
    OnlyBot
}
