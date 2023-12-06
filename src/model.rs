use serde::{Deserialize, Deserializer};
use serde::de::Error as DeserializeError;

use thiserror::Error;

pub type GameId = String;
pub type UserId = String;
pub type Fen = String;
pub type Rating = i32;
pub type Seconds = i32;
pub type Days = i32;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GameEventSource {
    Lobby,
    Friend,
    Ai,
    Api,
    Tournament,
    Position,
    Import,

    #[serde(rename = "importlive")]
    ImportLive,
    Simul,
    Relay,
    Pool,
    Swiss
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GameStatus {
    Created,
    Started,
    Aborted,
    Mate,
    Resign,
    Stalemate,
    Timeout,
    Draw,
    OutOfTime,
    Cheat,
    NoStart,
    UnknownFinish,
    VariantEnd
}

#[derive(Debug, Error)]
enum GameStatusError {

    #[error("unknown game status ID `{0}`")]
    UnknownId(i64),

    #[error("unknown game status name `{0}`")]
    UnknownName(String),

    #[error("game status ID `{0}` does not match name `{1}`")]
    IdNameMismatch(i64, String)
}

#[derive(Deserialize)]
struct GameStatusObject {
    // TODO id really optional?
    id: Option<i64>,
    name: Option<String>
}

fn game_status_from_id(id: i64) -> Result<GameStatus, GameStatusError> {
    match id {
        10 => Ok(GameStatus::Created),
        20 => Ok(GameStatus::Started),
        25 => Ok(GameStatus::Aborted),
        30 => Ok(GameStatus::Mate),
        31 => Ok(GameStatus::Resign),
        32 => Ok(GameStatus::Stalemate),
        33 => Ok(GameStatus::Timeout),
        34 => Ok(GameStatus::Draw),
        35 => Ok(GameStatus::OutOfTime),
        36 => Ok(GameStatus::Cheat),
        37 => Ok(GameStatus::NoStart),
        38 => Ok(GameStatus::UnknownFinish),
        60 => Ok(GameStatus::VariantEnd),
        _ => Err(GameStatusError::UnknownId(id))
    }
}

fn game_status_from_name(name: &String) -> Result<GameStatus, GameStatusError> {
    match name.as_str() {
        "created" => Ok(GameStatus::Created),
        "started" => Ok(GameStatus::Started),
        "aborted" => Ok(GameStatus::Aborted),
        "mate" => Ok(GameStatus::Mate),
        "resign" => Ok(GameStatus::Resign),
        "stalemate" => Ok(GameStatus::Stalemate),
        "timeout" => Ok(GameStatus::Timeout),
        "draw" => Ok(GameStatus::Draw),
        "outoftime" => Ok(GameStatus::OutOfTime),
        "cheat" => Ok(GameStatus::Cheat),
        "noStart" => Ok(GameStatus::NoStart),
        "unknownFinish" => Ok(GameStatus::UnknownFinish),
        "variantEnd" => Ok(GameStatus::VariantEnd),
        _ => Err(GameStatusError::UnknownName(name.to_owned()))
    }
}

impl TryFrom<GameStatusObject> for Option<GameStatus> {
    type Error = GameStatusError;

    fn try_from(game_status_object: GameStatusObject)
            -> Result<Option<GameStatus>, GameStatusError> {
        let from_id = game_status_object.id.map(game_status_from_id).transpose()?;
        let from_name = game_status_object.name.as_ref().map(game_status_from_name).transpose()?;

        if let (Some(from_id), Some(from_name)) = (from_id, from_name) {
            if from_id != from_name {
                return Err(GameStatusError::IdNameMismatch(
                    game_status_object.id.unwrap(), game_status_object.name.unwrap()));
            }
        }

        Ok(from_id.or(from_name))
    }
}

fn deserialize_game_status<'de, D>(deserializer: D) -> Result<Option<GameStatus>, D::Error>
where
    D: Deserializer<'de>
{
    let game_status_object = Option::<GameStatusObject>::deserialize(deserializer)?;

    match game_status_object {
        Some(game_status_object) =>
            Ok(game_status_object.try_into().map_err(DeserializeError::custom)?),
        None => Ok(None)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Player {
    White,
    Black
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct Compat {
    // TODO Option<bool> correct?
    bot: Option<bool>,
    board: Option<bool>
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct GameEventInfo {
    // TODO really so many options?
    pub id: Option<GameId>,
    pub source: Option<GameEventSource>,

    #[serde(deserialize_with = "deserialize_game_status")]
    pub status: Option<GameStatus>,
    pub winner: Option<Player>,
    pub compat: Option<Compat>
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeStatus {
    Created,
    Offline,
    Canceled,
    Declined,
    Accepted
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub enum Title {
    GM,
    WGM,
    IM,
    WIM,
    FM,
    WFM,
    NM,
    WNM,
    CM,
    WCM,
    LM,
    BOT
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct User {
    // TODO really so many options?
    pub rating: Option<Rating>,
    pub provisional: Option<bool>,
    pub online: Option<bool>,
    pub id: UserId,
    pub name: String,
    pub title: Option<Title>,
    pub patron: Option<bool>
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(tag = "key", rename_all = "camelCase")]
pub enum Variant {
    Standard,
    Chess960,
    Crazyhouse,
    Antichess,
    Atomic,
    Horde,
    KingOfTheHill,
    RacingKings,
    ThreeCheck,
    FromPosition
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Speed {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TimeControl {
    Clock {
        limit: Seconds,
        increment: Seconds
    },
    Correspondence {
        days_per_turn: Days
    },
    Unlimited
}

// TODO replace with Option<Player>?
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeColor {
    White,
    Black,
    Random
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeDirection {
    In,
    Out
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub id: GameId,
    pub url: String,
    pub status: ChallengeStatus,
    pub challenger: User,
    pub dest_user: Option<User>,
    pub variant: Variant,
    pub rated: bool,
    pub speed: Speed,
    pub time_control: TimeControl,
    pub color: ChallengeColor,
    pub direction: ChallengeDirection,
    pub initial_fen: Option<Fen>,
    pub decline_reason: Option<String>, // TODO unify with key?
    pub decline_reason_key: Option<String>
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct ChallengeDeclined {
    pub id: GameId
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Event {
    GameStart(GameEventInfo),
    GameFinish(GameEventInfo),
    Challenge(Challenge),
    ChallengeCancelled(Challenge),
    ChallengeDeclined(ChallengeDeclined)
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "camelCase")]
        enum Wrapper {
            GameStart {
                game: GameEventInfo
            },
            GameFinish {
                game: GameEventInfo
            },
            Challenge {
                challenge: Challenge
            },
            ChallengeCancelled {
                challenge: Challenge
            },
            ChallengeDeclined {
                challenge: ChallengeDeclined
            }
        }

        Ok(match Wrapper::deserialize(deserializer)? {
            Wrapper::GameStart { game } => Event::GameStart(game),
            Wrapper::GameFinish { game } => Event::GameFinish(game),
            Wrapper::Challenge { challenge } => Event::Challenge(challenge),
            Wrapper::ChallengeCancelled { challenge } => Event::ChallengeCancelled(challenge),
            Wrapper::ChallengeDeclined { challenge } => Event::ChallengeDeclined(challenge)
        })
    }
}
