use serde::Deserialize;

use crate::model::{Milliseconds, Seconds};
use crate::model::game::{Color, GameInfo, GameStatus};
use crate::model::game::chat::{ChatLine, ChatRoom};
use crate::model::user::{AiLevel, Rating, Title, UserId};

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
    #[serde(flatten)]
    pub info: GameInfo,
    pub state: GameStateEvent
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct ChatLineEvent {
    pub room: ChatRoom,

    #[serde(flatten)]
    pub chat_line: ChatLine
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OpponentGoneEvent {
    pub gone: bool,
    pub claim_win_in_seconds: Option<Seconds>
}

#[allow(clippy::large_enum_variant)] // TODO resolve this somehow
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

#[cfg(test)]
mod tests {

    use kernal::prelude::*;

    use rstest::rstest;

    use crate::model::game::{Clock, GamePerf, Speed, Variant};

    use super::*;

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
            info: GameInfo {
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
                tournament_id: None,
            },
            state: minimal_game_state_event()
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
            info: GameInfo {
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
                tournament_id: None,
            },
            state: minimal_game_state_event()
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
            info: GameInfo {
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
                tournament_id: None,
            },
            state: minimal_game_state_event()
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
            info: GameInfo {
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
                tournament_id: None
            },
            state: minimal_game_state_event()
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
            info: GameInfo {
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
                tournament_id: None
            },
            state: minimal_game_state_event(),
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
            info: GameInfo {
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
                tournament_id: None
            },
            state: minimal_game_state_event()
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
            info: GameInfo {
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
                tournament_id: Some("testTournamentId".to_owned())
            },
            state: minimal_game_state_event()
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
            chat_line: ChatLine {
                username: "testUsername".to_owned(),
                text: "testText".to_owned()
            }
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
