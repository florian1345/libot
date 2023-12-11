use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error as DeserializeError;

use thiserror::Error;

pub type GameId = String;
pub type UserId = String;
pub type TournamentId = String;
pub type Fen = String;
pub type Moves = String;
pub type Rating = i32;
pub type Milliseconds = i64;
pub type Seconds = i32;
pub type Days = i32;
pub type Timestamp = i64;
pub type AiLevel = i32;

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

fn deserialize_game_status_from_object<'de, D>(deserializer: D)
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Color {
    White,
    Black
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct Compat {
    // TODO Option<bool> correct?
    pub bot: Option<bool>,
    pub board: Option<bool>
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct GameStartFinishEvent {
    // TODO really so many options?
    pub id: Option<GameId>,
    pub source: Option<GameEventSource>,

    #[serde(default, deserialize_with = "deserialize_game_status_from_object")]
    pub status: Option<GameStatus>,
    pub winner: Option<Color>,
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

fn deserialize_optional_variant<'de, D>(deserializer: D) -> Result<Option<Variant>, D::Error>
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

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeEvent {
    pub id: GameId,
    pub url: String,
    pub status: ChallengeStatus,
    pub challenger: User,
    pub dest_user: Option<User>,

    // TODO really optional?
    #[serde(deserialize_with = "deserialize_optional_variant")]
    pub variant: Option<Variant>,
    pub rated: bool,
    pub speed: Speed,
    pub time_control: TimeControl,
    pub color: ChallengeColor,
    pub perf: ChallengePerf,
    pub direction: Option<ChallengeDirection>,
    pub initial_fen: Option<Fen>,
    pub decline_reason: Option<String>, // TODO unify with key?
    pub decline_reason_key: Option<DeclineReason>
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct ChallengeDeclinedEvent {
    pub id: GameId
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BotEvent {
    GameStart(GameStartFinishEvent),
    GameFinish(GameStartFinishEvent),
    Challenge(ChallengeEvent),
    ChallengeCanceled(ChallengeEvent),
    ChallengeDeclined(ChallengeDeclinedEvent)
}

impl<'de> Deserialize<'de> for BotEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "camelCase")]
        enum Wrapper {
            GameStart {
                game: GameStartFinishEvent
            },
            GameFinish {
                game: GameStartFinishEvent
            },
            Challenge {
                challenge: ChallengeEvent
            },
            ChallengeCanceled {
                challenge: ChallengeEvent
            },
            ChallengeDeclined {
                challenge: ChallengeDeclinedEvent
            }
        }

        Ok(match Wrapper::deserialize(deserializer)? {
            Wrapper::GameStart { game } => BotEvent::GameStart(game),
            Wrapper::GameFinish { game } => BotEvent::GameFinish(game),
            Wrapper::Challenge { challenge } => BotEvent::Challenge(challenge),
            Wrapper::ChallengeCanceled { challenge } => BotEvent::ChallengeCanceled(challenge),
            Wrapper::ChallengeDeclined { challenge } => BotEvent::ChallengeDeclined(challenge)
        })
    }
}

#[derive(Serialize)]
pub(crate) struct DeclineRequest {

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reason: Option<DeclineReason>
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct GamePerf {

    /// Translated perf name (e.g. "Classical" or "Blitz").
    pub name: Option<String>
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameEventPlayer {
    // TODO really everything optional?
    pub ai_level: Option<AiLevel>,
    pub id: Option<UserId>,
    pub name: Option<String>,
    pub title: Option<Title>,
    pub rating: Option<Rating>,
    pub provisional: Option<bool>
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct GameStateEvent {

    /// Current moves in UCI format.
    pub moves: String,

    /// Integer of milliseconds White has left on the clock.
    #[serde(rename = "wtime")]
    pub white_time: Milliseconds,

    /// Integer of milliseconds Black has left on the clock.
    #[serde(rename = "btime")]
    pub black_time: Milliseconds,

    // TODO milliseconds correct?

    /// Integer of White Fisher increment.
    #[serde(rename = "winc")]
    pub white_increment: Milliseconds,

    /// Integer of Black Fisher increment.
    #[serde(rename = "binc")]
    pub black_increment: Milliseconds,
    pub status: GameStatus,

    /// Color of the winner, if any.
    pub winner: Option<Color>,

    /// True if and only if White is offering a draw.
    #[serde(rename = "wdraw", default)]
    pub white_draw_offer: bool,

    /// True if and only if Black is offering a draw.
    #[serde(rename = "bdraw", default)]
    pub black_draw_offer: bool,

    /// True if and only if White is proposing a take-back.
    #[serde(rename = "wtakeback", default)]
    pub white_take_back_proposal: bool,

    /// True if and only if Black is proposing a take-back.
    #[serde(rename = "btakeback", default)]
    pub black_take_back_proposal: bool
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameFullEvent {
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
    pub state: GameStateEvent,
    pub tournament_id: Option<TournamentId>
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChatRoom {
    Player,
    Spectator
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct ChatLineEvent {
    pub room: ChatRoom,
    pub username: String,
    pub text: String
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OpponentGoneEvent {
    pub gone: bool,
    pub claim_win_in_seconds: Option<Seconds>
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GameEvent {

    /// Full game data. All values are immutable, except for the `state` field.
    GameFull(GameFullEvent),

    /// Current state of the game. Immutable values not included.
    GameState(GameStateEvent),

    /// Chat message sent by a user (or the bot itself) in the `room` "player" or "spectator".
    ChatLine(ChatLineEvent),

    /// Whether the opponent has left the game, and how long before you can claim a win or draw.
    OpponentGone(OpponentGoneEvent)
}

#[cfg(test)]
mod tests {

    use super::*;

    use kernal::prelude::*;

    use rstest::rstest;

    use serde_json::{Deserializer as JsonDeserializer, Result as JsonResult};

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

    fn minimal_user(id: &str, name: &str) -> User {
        User {
            rating: None,
            provisional: None,
            online: None,
            id: id.to_owned(),
            name: name.to_owned(),
            title: None,
            patron: None,
        }
    }

    #[rstest]
    #[case::game_start_minimal(
        r#"{
            "type": "gameStart",
            "game": { }
        }"#,
        BotEvent::GameStart(GameStartFinishEvent {
            id: None,
            source: None,
            status: None,
            winner: None,
            compat: None
        })
    )]
    #[case::game_start_with_id(
        r#"{
            "type": "gameStart",
            "game": {
                "id": "test"
            }
        }"#,
        BotEvent::GameStart(GameStartFinishEvent {
            id: Some("test".to_owned()),
            source: None,
            status: None,
            winner: None,
            compat: None
        })
    )]
    #[case::game_start_with_source(
        r#"{
            "type": "gameStart",
            "game": {
                "source": "friend"
            }
        }"#,
        BotEvent::GameStart(GameStartFinishEvent {
            id: None,
            source: Some(GameEventSource::Friend),
            status: None,
            winner: None,
            compat: None
        })
    )]
    #[case::game_start_with_status(
        r#"{
            "type": "gameStart",
            "game": {
                "status": {
                    "id": 10,
                    "name": "created"
                }
            }
        }"#,
        BotEvent::GameStart(GameStartFinishEvent {
            id: None,
            source: None,
            status: Some(GameStatus::Created),
            winner: None,
            compat: None
        })
    )]
    #[case::game_start_with_winner(
        r#"{
            "type": "gameStart",
            "game": {
                "winner": "white"
            }
        }"#,
        BotEvent::GameStart(GameStartFinishEvent {
            id: None,
            source: None,
            status: None,
            winner: Some(Color::White),
            compat: None
        })
    )]
    #[case::game_start_with_empty_compat(
        r#"{
            "type": "gameStart",
            "game": {
                "compat": { }
            }
        }"#,
        BotEvent::GameStart(GameStartFinishEvent {
            id: None,
            source: None,
            status: None,
            winner: None,
            compat: Some(Compat {
                board: None,
                bot: None
            })
        })
    )]
    #[case::game_finish_with_non_empty_compat(
        r#"{
            "type": "gameFinish",
            "game": {
                "compat": {
                    "board": true,
                    "bot": false
                }
            }
        }"#,
        BotEvent::GameFinish(GameStartFinishEvent {
            id: None,
            source: None,
            status: None,
            winner: None,
            compat: Some(Compat {
                board: Some(true),
                bot: Some(false)
            })
        })
    )]
    #[case::minimal_challenge(
        r#"{
            "type": "challenge",
            "challenge": {
                "id": "testId",
                "url": "testUrl",
                "status": "created",
                "challenger": {
                    "id": "testChallengerId",
                    "name": "testChallengerName"
                },
                "destUser": null,
                "variant": { },
                "rated": false,
                "speed": "rapid",
                "timeControl": {
                    "type": "unlimited"
                },
                "color": "random",
                "perf": { }
            }
        }"#,
        BotEvent::Challenge(ChallengeEvent {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: User {
                rating: None,
                provisional: None,
                online: None,
                id: "testChallengerId".to_owned(),
                name: "testChallengerName".to_owned(),
                title: None,
                patron: None
            },
            dest_user: None,
            variant: None,
            rated: false,
            speed: Speed::Rapid,
            time_control: TimeControl::Unlimited,
            color: ChallengeColor::Random,
            perf: ChallengePerf {
                icon: None,
                name: None
            },
            direction: None,
            initial_fen: None,
            decline_reason: None,
            decline_reason_key: None
        })
    )]
    #[case::challenge_with_full_challenger(
        r#"{
            "type": "challenge",
            "challenge": {
                "id": "testId",
                "url": "testUrl",
                "status": "created",
                "challenger": {
                    "rating": 2345,
                    "provisional": false,
                    "online": true,
                    "id": "testChallengerId",
                    "name": "testChallengerName",
                    "title": "WGM",
                    "patron": true
                },
                "destUser": null,
                "variant": { },
                "rated": true,
                "speed": "blitz",
                "timeControl": {
                    "type": "unlimited"
                },
                "color": "white",
                "perf": { }
            }
        }"#,
        BotEvent::Challenge(ChallengeEvent {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: User {
                rating: Some(2345),
                provisional: Some(false),
                online: Some(true),
                id: "testChallengerId".to_owned(),
                name: "testChallengerName".to_owned(),
                title: Some(Title::Wgm),
                patron: Some(true)
            },
            dest_user: None,
            variant: None,
            rated: true,
            speed: Speed::Blitz,
            time_control: TimeControl::Unlimited,
            color: ChallengeColor::White,
            perf: ChallengePerf {
                icon: None,
                name: None
            },
            direction: None,
            initial_fen: None,
            decline_reason: None,
            decline_reason_key: None
        })
    )]
    #[case::challenge_with_dest_user(
        r#"{
            "type": "challenge",
            "challenge": {
                "id": "testId",
                "url": "testUrl",
                "status": "created",
                "challenger": {
                    "id": "testChallengerId",
                    "name": "testChallengerName"
                },
                "destUser": {
                    "id": "testDestUserId",
                    "name": "testDestUserName"
                },
                "variant": { },
                "rated": false,
                "speed": "classical",
                "timeControl": {
                    "type": "unlimited"
                },
                "color": "black",
                "perf": { }
            }
        }"#,
        BotEvent::Challenge(ChallengeEvent {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: User {
                rating: None,
                provisional: None,
                online: None,
                id: "testChallengerId".to_owned(),
                name: "testChallengerName".to_owned(),
                title: None,
                patron: None
            },
            dest_user: Some(User {
                rating: None,
                provisional: None,
                online: None,
                id: "testDestUserId".to_owned(),
                name: "testDestUserName".to_owned(),
                title: None,
                patron: None
            }),
            variant: None,
            rated: false,
            speed: Speed::Classical,
            time_control: TimeControl::Unlimited,
            color: ChallengeColor::Black,
            perf: ChallengePerf {
                icon: None,
                name: None
            },
            direction: None,
            initial_fen: None,
            decline_reason: None,
            decline_reason_key: None
        })
    )]
    #[case::challenge_with_full_variant(
        r#"{
            "type": "challenge",
            "challenge": {
                "id": "testId",
                "url": "testUrl",
                "status": "created",
                "challenger": {
                    "id": "testChallengerId",
                    "name": "testChallengerName"
                },
                "variant": {
                    "key": "chess960",
                    "name": "Chess 960",
                    "short": "C960"
                },
                "rated": false,
                "speed": "bullet",
                "timeControl": {
                    "type": "unlimited"
                },
                "color": "random",
                "perf": { }
            }
        }"#,
        BotEvent::Challenge(ChallengeEvent {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: minimal_user("testChallengerId", "testChallengerName"),
            dest_user: None,
            variant: Some(Variant::Chess960),
            rated: false,
            speed: Speed::Bullet,
            time_control: TimeControl::Unlimited,
            color: ChallengeColor::Random,
            perf: ChallengePerf {
                icon: None,
                name: None
            },
            direction: None,
            initial_fen: None,
            decline_reason: None,
            decline_reason_key: None
        })
    )]
    #[case::challenge_with_filled_perf(
        r#"{
            "type": "challenge",
            "challenge": {
                "id": "testId",
                "url": "testUrl",
                "status": "created",
                "challenger": {
                    "id": "testChallengerId",
                    "name": "testChallengerName"
                },
                "variant": { },
                "rated": true,
                "speed": "ultraBullet",
                "timeControl": {
                    "type": "unlimited"
                },
                "color": "black",
                "perf": {
                    "icon": "testIcon",
                    "name": "testName"
                }
            }
        }"#,
        BotEvent::Challenge(ChallengeEvent {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: minimal_user("testChallengerId", "testChallengerName"),
            dest_user: None,
            variant: None,
            rated: true,
            speed: Speed::UltraBullet,
            time_control: TimeControl::Unlimited,
            color: ChallengeColor::Black,
            perf: ChallengePerf {
                icon: Some("testIcon".to_owned()),
                name: Some("testName".to_owned())
            },
            direction: None,
            initial_fen: None,
            decline_reason: None,
            decline_reason_key: None
        })
    )]
    #[case::challenge_canceled_with_remaining_optional_strings(
        r#"{
            "type": "challengeCanceled",
            "challenge": {
                "id": "testId",
                "url": "testUrl",
                "status": "created",
                "challenger": {
                    "id": "testChallengerId",
                    "name": "testChallengerName"
                },
                "variant": { },
                "rated": false,
                "speed": "rapid",
                "timeControl": {
                    "type": "unlimited"
                },
                "color": "white",
                "perf": { },
                "direction": "in",
                "initialFen": "testFen",
                "declineReason": "testDeclineReason",
                "declineReasonKey": "noBot"
            }
        }"#,
        BotEvent::ChallengeCanceled(ChallengeEvent {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: minimal_user("testChallengerId", "testChallengerName"),
            dest_user: None,
            variant: None,
            rated: false,
            speed: Speed::Rapid,
            time_control: TimeControl::Unlimited,
            color: ChallengeColor::White,
            perf: ChallengePerf {
                icon: None,
                name: None
            },
            direction: Some(ChallengeDirection::In),
            initial_fen: Some("testFen".to_owned()),
            decline_reason: Some("testDeclineReason".to_owned()),
            decline_reason_key: Some(DeclineReason::NoBot)
        })
    )]
    #[case::challenge_canceled_with_clock_time_control(
        r#"{
            "type": "challengeCanceled",
            "challenge": {
                "id": "testId",
                "url": "testUrl",
                "status": "created",
                "challenger": {
                    "id": "testChallengerId",
                    "name": "testChallengerName"
                },
                "variant": { },
                "rated": true,
                "speed": "blitz",
                "timeControl": {
                    "type": "clock",
                    "limit": 300,
                    "increment": 3,
                    "show": "5+3"
                },
                "color": "black",
                "perf": { }
            }
        }"#,
        BotEvent::ChallengeCanceled(ChallengeEvent {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: minimal_user("testChallengerId", "testChallengerName"),
            dest_user: None,
            variant: None,
            rated: true,
            speed: Speed::Blitz,
            time_control: TimeControl::Clock(Clock {
                limit: Some(300),
                increment: Some(3)
            }),
            color: ChallengeColor::Black,
            perf: ChallengePerf {
                icon: None,
                name: None
            },
            direction: None,
            initial_fen: None,
            decline_reason: None,
            decline_reason_key: None
        })
    )]
    #[case::challenge_canceled_with_clock_time_control(
        r#"{
            "type": "challengeCanceled",
            "challenge": {
                "id": "testId",
                "url": "testUrl",
                "status": "created",
                "challenger": {
                    "id": "testChallengerId",
                    "name": "testChallengerName"
                },
                "variant": { },
                "rated": false,
                "speed": "correspondence",
                "timeControl": {
                    "type": "correspondence",
                    "daysPerTurn": 2
                },
                "color": "random",
                "perf": { }
            }
        }"#,
        BotEvent::ChallengeCanceled(ChallengeEvent {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: minimal_user("testChallengerId", "testChallengerName"),
            dest_user: None,
            variant: None,
            rated: false,
            speed: Speed::Correspondence,
            time_control: TimeControl::Correspondence {
                days_per_turn: Some(2)
            },
            color: ChallengeColor::Random,
            perf: ChallengePerf {
                icon: None,
                name: None
            },
            direction: None,
            initial_fen: None,
            decline_reason: None,
            decline_reason_key: None
        })
    )]
    #[case::challenge_declined(
        r#"{
            "type": "challengeDeclined",
            "challenge": {
                "id": "testId"
            }
        }"#,
        BotEvent::ChallengeDeclined(ChallengeDeclinedEvent {
            id: "testId".to_owned()
        })
    )]
    fn parse_bot_event(#[case] json: &str, #[case] expected_event: BotEvent) {
        let event = serde_json::from_str(json).unwrap();

        assert_that!(event).is_equal_to(expected_event);
    }

    #[test]
    fn serialize_decline_request_without_reason() {
        let decline_request = DeclineRequest {
            reason: None
        };

        let serialized = serde_json::to_string(&decline_request).unwrap();

        assert_that!(serialized).is_equal_to("{}".to_owned());
    }

    #[rstest]
    #[case(DeclineReason::Generic, r#"{"reason":"generic"}"#)]
    #[case(DeclineReason::Later, r#"{"reason":"later"}"#)]
    #[case(DeclineReason::TooFast, r#"{"reason":"tooFast"}"#)]
    #[case(DeclineReason::TooSlow, r#"{"reason":"tooSlow"}"#)]
    #[case(DeclineReason::TimeControl, r#"{"reason":"timeControl"}"#)]
    #[case(DeclineReason::Rated, r#"{"reason":"rated"}"#)]
    #[case(DeclineReason::Casual, r#"{"reason":"casual"}"#)]
    #[case(DeclineReason::Standard, r#"{"reason":"standard"}"#)]
    #[case(DeclineReason::Variant, r#"{"reason":"variant"}"#)]
    #[case(DeclineReason::NoBot, r#"{"reason":"noBot"}"#)]
    #[case(DeclineReason::OnlyBot, r#"{"reason":"onlyBot"}"#)]
    fn serialize_decline_request_with_reason(
            #[case] reason: DeclineReason, #[case] expected_json: &str) {
        let decline_request = DeclineRequest {
            reason: Some(reason)
        };

        let serialized = serde_json::to_string(&decline_request).unwrap();

        assert_that!(serialized).is_equal_to(expected_json.to_owned());
    }

    fn empty_game_event_player() -> GameEventPlayer {
        GameEventPlayer {
            ai_level: None,
            id: None,
            name: None,
            title: None,
            rating: None,
            provisional: None
        }
    }

    fn minimal_game_state_event() -> GameStateEvent {
        GameStateEvent {
            moves: "testMoves".to_owned(),
            white_time: 100,
            black_time: 200,
            white_increment: 1,
            black_increment: 2,
            status: GameStatus::Created,
            winner: None,
            white_draw_offer: false,
            black_draw_offer: false,
            white_take_back_proposal: false,
            black_take_back_proposal: false
        }
    }

    #[rstest]
    #[case::minimal_game_full(
        r#"{
            "type": "gameFull",
            "id": "testId",
            "variant": { },
            "clock": null,
            "speed": "blitz",
            "perf": { },
            "rated": true,
            "createdAt": 1234,
            "white": { },
            "black": { },
            "initialFen": "testInitialFen",
            "state": {
                "type": "gameState",
                "moves": "testMoves",
                "wtime": 100,
                "btime": 200,
                "winc": 1,
                "binc": 2,
                "status": "created"
            }
        }"#,
        GameEvent::GameFull(GameFullEvent {
            id: "testId".to_owned(),
            variant: None,
            clock: None,
            speed: Speed::Blitz,
            perf: GamePerf {
                name: None
            },
            rated: true,
            created_at: 1234,
            white: empty_game_event_player(),
            black: empty_game_event_player(),
            initial_fen: "testInitialFen".to_owned(),
            state: minimal_game_state_event(),
            tournament_id: None
        })
    )]
    #[case::game_full_with_variant(
        r#"{
            "type": "gameFull",
            "id": "testId",
            "variant": {
                "key": "crazyhouse",
                "name": "Crazyhouse",
                "short": "CH"
            },
            "clock": null,
            "speed": "rapid",
            "perf": { },
            "rated": false,
            "createdAt": 1234,
            "white": { },
            "black": { },
            "initialFen": "testInitialFen",
            "state": {
                "type": "gameState",
                "moves": "testMoves",
                "wtime": 100,
                "btime": 200,
                "winc": 1,
                "binc": 2,
                "status": "created"
            }
        }"#,
        GameEvent::GameFull(GameFullEvent {
            id: "testId".to_owned(),
            variant: Some(Variant::Crazyhouse),
            clock: None,
            speed: Speed::Rapid,
            perf: GamePerf {
                name: None
            },
            rated: false,
            created_at: 1234,
            white: empty_game_event_player(),
            black: empty_game_event_player(),
            initial_fen: "testInitialFen".to_owned(),
            state: minimal_game_state_event(),
            tournament_id: None
        })
    )]
    #[case::game_full_with_empty_clock(
        r#"{
            "type": "gameFull",
            "id": "testId",
            "variant": { },
            "clock": { },
            "speed": "classical",
            "perf": { },
            "rated": true,
            "createdAt": 1234,
            "white": { },
            "black": { },
            "initialFen": "testInitialFen",
            "state": {
                "type": "gameState",
                "moves": "testMoves",
                "wtime": 100,
                "btime": 200,
                "winc": 1,
                "binc": 2,
                "status": "created"
            }
        }"#,
        GameEvent::GameFull(GameFullEvent {
            id: "testId".to_owned(),
            variant: None,
            clock: Some(Clock {
                limit: None,
                increment: None
            }),
            speed: Speed::Classical,
            perf: GamePerf {
                name: None
            },
            rated: true,
            created_at: 1234,
            white: empty_game_event_player(),
            black: empty_game_event_player(),
            initial_fen: "testInitialFen".to_owned(),
            state: minimal_game_state_event(),
            tournament_id: None
        })
    )]
    #[case::game_full_with_full_clock(
        r#"{
            "type": "gameFull",
            "id": "testId",
            "variant": { },
            "clock": {
                "limit": 60,
                "increment": 1
            },
            "speed": "bullet",
            "perf": { },
            "rated": true,
            "createdAt": 1234,
            "white": { },
            "black": { },
            "initialFen": "testInitialFen",
            "state": {
                "type": "gameState",
                "moves": "testMoves",
                "wtime": 100,
                "btime": 200,
                "winc": 1,
                "binc": 2,
                "status": "created"
            }
        }"#,
        GameEvent::GameFull(GameFullEvent {
            id: "testId".to_owned(),
            variant: None,
            clock: Some(Clock {
                limit: Some(60),
                increment: Some(1)
            }),
            speed: Speed::Bullet,
            perf: GamePerf {
                name: None
            },
            rated: true,
            created_at: 1234,
            white: empty_game_event_player(),
            black: empty_game_event_player(),
            initial_fen: "testInitialFen".to_owned(),
            state: minimal_game_state_event(),
            tournament_id: None
        })
    )]
    #[case::game_full_with_perf(
        r#"{
            "type": "gameFull",
            "id": "testId",
            "variant": { },
            "clock": null,
            "speed": "ultraBullet",
            "perf": {
                "name": "testPerfName"
            },
            "rated": true,
            "createdAt": 1234,
            "white": { },
            "black": { },
            "initialFen": "testInitialFen",
            "state": {
                "type": "gameState",
                "moves": "testMoves",
                "wtime": 100,
                "btime": 200,
                "winc": 1,
                "binc": 2,
                "status": "created"
            }
        }"#,
        GameEvent::GameFull(GameFullEvent {
            id: "testId".to_owned(),
            variant: None,
            clock: None,
            speed: Speed::UltraBullet,
            perf: GamePerf {
                name: Some("testPerfName".to_owned())
            },
            rated: true,
            created_at: 1234,
            white: empty_game_event_player(),
            black: empty_game_event_player(),
            initial_fen: "testInitialFen".to_owned(),
            state: minimal_game_state_event(),
            tournament_id: None
        })
    )]
    #[case::game_full_with_players(
        r#"{
            "type": "gameFull",
            "id": "testId",
            "variant": { },
            "clock": null,
            "speed": "blitz",
            "perf": { },
            "rated": true,
            "createdAt": 1234,
            "white": {
                "aiLevel": 5,
                "id": "testWhiteId",
                "name": "testWhiteName",
                "title": null,
                "rating": 2000,
                "provisional": true
            },
            "black": {
                "id": "testBlackId",
                "name": "testBlackName",
                "title": "IM",
                "rating": 2145,
                "provisional": false
            },
            "initialFen": "testInitialFen",
            "state": {
                "type": "gameState",
                "moves": "testMoves",
                "wtime": 100,
                "btime": 200,
                "winc": 1,
                "binc": 2,
                "status": "created"
            }
        }"#,
        GameEvent::GameFull(GameFullEvent {
            id: "testId".to_owned(),
            variant: None,
            clock: None,
            speed: Speed::Blitz,
            perf: GamePerf {
                name: None
            },
            rated: true,
            created_at: 1234,
            white: GameEventPlayer {
                ai_level: Some(5),
                id: Some("testWhiteId".to_owned()),
                name: Some("testWhiteName".to_owned()),
                title: None,
                rating: Some(2000),
                provisional: Some(true)
            },
            black: GameEventPlayer {
                ai_level: None,
                id: Some("testBlackId".to_owned()),
                name: Some("testBlackName".to_owned()),
                title: Some(Title::Im),
                rating: Some(2145),
                provisional: Some(false)
            },
            initial_fen: "testInitialFen".to_owned(),
            state: minimal_game_state_event(),
            tournament_id: None
        })
    )]
    #[case::game_full_with_tournament_id(
        r#"{
            "type": "gameFull",
            "id": "testId",
            "variant": { },
            "clock": null,
            "speed": "correspondence",
            "perf": { },
            "rated": false,
            "createdAt": 1234,
            "white": { },
            "black": { },
            "initialFen": "testInitialFen",
            "state": {
                "type": "gameState",
                "moves": "testMoves",
                "wtime": 100,
                "btime": 200,
                "winc": 1,
                "binc": 2,
                "status": "created"
            },
            "tournamentId": "testTournamentId"
        }"#,
        GameEvent::GameFull(GameFullEvent {
            id: "testId".to_owned(),
            variant: None,
            clock: None,
            speed: Speed::Correspondence,
            perf: GamePerf {
                name: None
            },
            rated: false,
            created_at: 1234,
            white: empty_game_event_player(),
            black: empty_game_event_player(),
            initial_fen: "testInitialFen".to_owned(),
            state: minimal_game_state_event(),
            tournament_id: Some("testTournamentId".to_owned())
        })
    )]
    #[case::minimal_game_state(
        r#"{
            "type": "gameState",
            "moves": "testMoves",
            "wtime": 15000,
            "btime": 19000,
            "winc": 1000,
            "binc": 1500,
            "status": "started"
        }"#,
        GameEvent::GameState(GameStateEvent {
            moves: "testMoves".to_owned(),
            white_time: 15000,
            black_time: 19000,
            white_increment: 1000,
            black_increment: 1500,
            status: GameStatus::Started,
            winner: None,
            white_draw_offer: false,
            black_draw_offer: false,
            white_take_back_proposal: false,
            black_take_back_proposal: false
        })
    )]
    #[case::game_state_with_winner(
        r#"{
            "type": "gameState",
            "moves": "testMoves",
            "wtime": 15000,
            "btime": 19000,
            "winc": 1000,
            "binc": 1500,
            "status": "mate",
            "winner": "black"
        }"#,
        GameEvent::GameState(GameStateEvent {
            moves: "testMoves".to_owned(),
            white_time: 15000,
            black_time: 19000,
            white_increment: 1000,
            black_increment: 1500,
            status: GameStatus::Mate,
            winner: Some(Color::Black),
            white_draw_offer: false,
            black_draw_offer: false,
            white_take_back_proposal: false,
            black_take_back_proposal: false
        })
    )]
    #[case::game_state_with_draw_offers(
        r#"{
            "type": "gameState",
            "moves": "testMoves",
            "wtime": 15000,
            "btime": 19000,
            "winc": 1000,
            "binc": 1500,
            "status": "draw",
            "wdraw": true,
            "bdraw": true
        }"#,
        GameEvent::GameState(GameStateEvent {
            moves: "testMoves".to_owned(),
            white_time: 15000,
            black_time: 19000,
            white_increment: 1000,
            black_increment: 1500,
            status: GameStatus::Draw,
            winner: None,
            white_draw_offer: true,
            black_draw_offer: true,
            white_take_back_proposal: false,
            black_take_back_proposal: false
        })
    )]
    #[case::game_state_with_draw_takebacks(
        r#"{
            "type": "gameState",
            "moves": "testMoves",
            "wtime": 15000,
            "btime": 19000,
            "winc": 1000,
            "binc": 1500,
            "status": "draw",
            "wtakeback": true,
            "btakeback": true
        }"#,
        GameEvent::GameState(GameStateEvent {
            moves: "testMoves".to_owned(),
            white_time: 15000,
            black_time: 19000,
            white_increment: 1000,
            black_increment: 1500,
            status: GameStatus::Draw,
            winner: None,
            white_draw_offer: false,
            black_draw_offer: false,
            white_take_back_proposal: true,
            black_take_back_proposal: true
        })
    )]
    #[case::chat_line(
        r#"{
            "type": "chatLine",
            "room": "spectator",
            "username": "testUsername",
            "text": "testText"
        }"#,
        GameEvent::ChatLine(ChatLineEvent {
            room: ChatRoom::Spectator,
            username: "testUsername".to_owned(),
            text: "testText".to_owned()
        })
    )]
    #[case::minimal_opponent_gone(
        r#"{
            "type": "opponentGone",
            "gone": false
        }"#,
        GameEvent::OpponentGone(OpponentGoneEvent {
            gone: false,
            claim_win_in_seconds: None
        })
    )]
    #[case::opponent_gone_with_claim_win_in_seconds(
        r#"{
            "type": "opponentGone",
            "gone": false,
            "claimWinInSeconds": 15
        }"#,
        GameEvent::OpponentGone(OpponentGoneEvent {
            gone: false,
            claim_win_in_seconds: Some(15)
        })
    )]
    fn parse_game_event(#[case] json: &str, #[case] expected_event: GameEvent) {
        let event = serde_json::from_str(json).unwrap();

        assert_that!(event).is_equal_to(expected_event);
    }
}
