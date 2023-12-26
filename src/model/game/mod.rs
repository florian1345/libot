use serde::{Deserialize, Deserializer};
use serde::de::Error as DeserializeError;

use thiserror::Error;

use crate::model::{Seconds, Timestamp};
use crate::model::game::event::GameEventPlayer;

pub mod chat;
pub mod event;

pub type GameId = String;
pub type TournamentId = String;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GameStatus {
    Created,
    Started,
    Aborted,
    Mate,
    Resign,
    Stalemate,
    Timeout,
    Draw,

    #[serde(rename = "outoftime")]
    OutOfTime,
    Cheat,
    NoStart,
    UnknownFinish,
    VariantEnd
}

impl GameStatus {

    /// Indicates whether a game with this status is running, i.e. no decision has been reached and
    /// moves can still be played.
    ///
    /// # Returns
    ///
    /// `true` if and only if a game with this status is running.
    pub fn is_running(self) -> bool {
        matches!(self, GameStatus::Created | GameStatus::Started)
    }
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

pub(crate) fn deserialize_game_status_from_object<'de, D>(deserializer: D)
    -> Result<Option<GameStatus>, D::Error>
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

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct GamePerf {

    /// Translated perf name (e.g. "Classical" or "Blitz").
    pub name: Option<String>
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub id: GameId,

    // TODO really optional?
    #[serde(deserialize_with = "deserialize_optional_variant")]
    pub variant: Option<Variant>,
    pub clock: Option<Clock>,
    pub speed: Speed,
    pub perf: GamePerf,
    pub rated: bool,
    pub created_at: Timestamp,
    pub white: GameEventPlayer,
    pub black: GameEventPlayer,
    pub initial_fen: Fen,
    pub tournament_id: Option<TournamentId>
}

// TODO avoid expensive clone with IDs?
pub type Fen = String;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Color {
    White,
    Black
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

pub(crate) fn deserialize_optional_variant<'de, D>(deserializer: D) -> Result<Option<Variant>, D::Error>
where
    D: Deserializer<'de>
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum VariantOrEmpty {
        Variant(Variant),
        Empty { }
    }

    let variant_or_empty = VariantOrEmpty::deserialize(deserializer)?;

    match variant_or_empty {
        VariantOrEmpty::Variant(variant) => Ok(Some(variant)),
        VariantOrEmpty::Empty { } => Ok(None)
    }
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
pub struct Clock {
    // TODO really optional?
    pub limit: Option<Seconds>,
    pub increment: Option<Seconds>
}

#[cfg(test)]
mod tests {

    use kernal::prelude::*;

    use rstest::rstest;

    use serde_json::{Deserializer as JsonDeserializer, Result as JsonResult};

    use crate::model::game::{deserialize_game_status_from_object, GameStatus};

    fn parse_game_status(json: &str) -> JsonResult<Option<GameStatus>> {
        let mut deserializer = JsonDeserializer::from_str(&json);
        deserialize_game_status_from_object(&mut deserializer)
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

    #[rstest]
    #[case::created(GameStatus::Created, true)]
    #[case::started(GameStatus::Started, true)]
    #[case::aborted(GameStatus::Aborted, false)]
    #[case::mate(GameStatus::Mate, false)]
    #[case::resign(GameStatus::Resign, false)]
    #[case::stalemate(GameStatus::Stalemate, false)]
    #[case::timeout(GameStatus::Timeout, false)]
    #[case::draw(GameStatus::Draw, false)]
    #[case::out_of_time(GameStatus::OutOfTime, false)]
    #[case::cheat(GameStatus::Cheat, false)]
    #[case::no_start(GameStatus::NoStart, false)]
    #[case::unknown_finish(GameStatus::UnknownFinish, false)]
    #[case::variant_end(GameStatus::VariantEnd, false)]
    fn game_status_is_running(#[case] game_status: GameStatus, #[case] expected_is_running: bool) {
        assert_that!(game_status.is_running()).is_equal_to(expected_is_running);
    }
}
