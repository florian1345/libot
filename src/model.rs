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
#[serde(rename_all = "UPPERCASE")]
pub enum Title {
    Gm,
    Wgm,
    Im,
    Wim,
    Fm,
    Wfm,
    Nm,
    Wnm,
    Cm,
    Wcm,
    Lm,
    Bot
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

#[cfg(test)]
mod tests {

    use super::*;

    use kernal::prelude::*;

    use rstest::rstest;

    use serde_json::{Deserializer as JsonDeserializer, Result as JsonResult};

    fn parse_game_status(json: &str) -> JsonResult<Option<GameStatus>> {
        let mut deserializer = JsonDeserializer::from_str(&json);
        deserialize_game_status(&mut deserializer)
    }

    #[rstest]
    #[case::created(10, GameStatus::Created)]
    #[case::started(20, GameStatus::Started)]
    #[case::aborted(25, GameStatus::Aborted)]
    #[case::mate(30, GameStatus::Mate)]
    #[case::resign(31, GameStatus::Resign)]
    #[case::stalemate(32, GameStatus::Stalemate)]
    #[case::timeout(33, GameStatus::Timeout)]
    #[case::draw(34, GameStatus::Draw)]
    #[case::out_of_time(35, GameStatus::OutOfTime)]
    #[case::cheat(36, GameStatus::Cheat)]
    #[case::no_start(37, GameStatus::NoStart)]
    #[case::unknown_finish(38, GameStatus::UnknownFinish)]
    #[case::variant_end(60, GameStatus::VariantEnd)]
    fn parse_game_status_works_for_id_only(#[case] id: i64, #[case] expected_status: GameStatus) {
        let json = format!("{{\"id\":{}}}", id);

        let status = parse_game_status(&json).unwrap();

        assert_that!(status).contains(expected_status);
    }

    #[rstest]
    #[case::created("created", GameStatus::Created)]
    #[case::started("started", GameStatus::Started)]
    #[case::aborted("aborted", GameStatus::Aborted)]
    #[case::mate("mate", GameStatus::Mate)]
    #[case::resign("resign", GameStatus::Resign)]
    #[case::stalemate("stalemate", GameStatus::Stalemate)]
    #[case::timeout("timeout", GameStatus::Timeout)]
    #[case::draw("draw", GameStatus::Draw)]
    #[case::out_of_time("outoftime", GameStatus::OutOfTime)]
    #[case::cheat("cheat", GameStatus::Cheat)]
    #[case::no_start("noStart", GameStatus::NoStart)]
    #[case::unknown_finish("unknownFinish", GameStatus::UnknownFinish)]
    #[case::variant_end("variantEnd", GameStatus::VariantEnd)]
    fn parse_game_status_works_for_name_only(#[case] name: &str, #[case] expected_status: GameStatus) {
        let json = format!("{{\"name\":\"{}\"}}", name);

        let status = parse_game_status(&json).unwrap();

        assert_that!(status).contains(expected_status);
    }

    #[rstest]
    #[case::created(10, "created", GameStatus::Created)]
    #[case::started(20, "started", GameStatus::Started)]
    #[case::aborted(25, "aborted", GameStatus::Aborted)]
    #[case::mate(30, "mate", GameStatus::Mate)]
    #[case::resign(31, "resign", GameStatus::Resign)]
    #[case::stalemate(32, "stalemate", GameStatus::Stalemate)]
    #[case::timeout(33, "timeout", GameStatus::Timeout)]
    #[case::draw(34, "draw", GameStatus::Draw)]
    #[case::out_of_time(35, "outoftime", GameStatus::OutOfTime)]
    #[case::cheat(36, "cheat", GameStatus::Cheat)]
    #[case::no_start(37, "noStart", GameStatus::NoStart)]
    #[case::unknown_finish(38, "unknownFinish", GameStatus::UnknownFinish)]
    #[case::variant_end(60, "variantEnd", GameStatus::VariantEnd)]
    fn parse_game_status_works_for_id_and_name(
            #[case] id: i64,
            #[case] name: &str,
            #[case] expected_status: GameStatus) {
        let json = format!("{{\"id\":{},\"name\":\"{}\"}}", id, name);

        let status = parse_game_status(&json).unwrap();

        assert_that!(status).contains(expected_status);
    }

    #[rstest]
    #[case::unknown_id("{\"id\":5}")]
    #[case::unknown_name("{\"name\":\"help\"}")]
    #[case::mismatch("{\"id\":10,\"name\":\"aborted\"}")]
    fn parse_game_status_fails(#[case] json: &str) {
        let status = parse_game_status(&json);

        assert_that!(status).is_err();
    }

    #[rstest]
    #[case::null("null")]
    #[case::empty("{}")]
    #[case::null_id("{\"id\":null}")]
    #[case::null_name("{\"name\":null}")]
    #[case::null_id_and_name("{\"id\":null,\"name\":null}")]
    fn parse_game_status_is_none(#[case] json: &str) {
        let status = parse_game_status(&json).unwrap();

        assert_that!(status).is_none();
    }
}
