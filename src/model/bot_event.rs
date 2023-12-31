use serde::{Deserialize, Deserializer};

use crate::model::challenge::{Challenge, ChallengeDeclined};
use crate::model::Compat;
use crate::model::game::event::GameEventSource;
use crate::model::game::{
    Color,
    deserialize_game_status_from_object,
    GameId,
    GameStatus
};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct GameStartFinish {
    // TODO really so many options?
    pub id: Option<GameId>,
    pub source: Option<GameEventSource>,

    #[serde(default, deserialize_with = "deserialize_game_status_from_object")]
    pub status: Option<GameStatus>,
    pub winner: Option<Color>,
    pub compat: Option<Compat>
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BotEvent {
    GameStart(GameStartFinish),
    GameFinish(GameStartFinish),
    Challenge(Challenge),
    ChallengeCanceled(Challenge),
    ChallengeDeclined(ChallengeDeclined)
}

impl<'de> Deserialize<'de> for BotEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "camelCase")]
        enum Wrapper {
            GameStart {
                game: GameStartFinish
            },
            GameFinish {
                game: GameStartFinish
            },
            Challenge {
                challenge: Challenge
            },
            ChallengeCanceled {
                challenge: Challenge
            },
            ChallengeDeclined {
                challenge: ChallengeDeclined
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

#[cfg(test)]
mod tests {

    use kernal::prelude::*;

    use rstest::rstest;

    use crate::model::TimeControl;
    use crate::model::challenge::{
        ChallengeColor,
        ChallengeDirection,
        ChallengePerf,
        ChallengeStatus,
        DeclineReason
    };
    use crate::model::game::{Clock, Speed, Variant};
    use crate::model::user::{Title, User};

    use super::*;

    fn minimal_user(id: &str, name: &str) -> User {
        User {
            rating: None,
            provisional: false,
            online: false,
            id: id.to_owned(),
            name: name.to_owned(),
            title: None,
            patron: false
        }
    }

    #[rstest]
    #[case::game_start_minimal(
        r#"{
            "type": "gameStart",
            "game": { }
        }"#,
        BotEvent::GameStart(GameStartFinish {
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
        BotEvent::GameStart(GameStartFinish {
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
        BotEvent::GameStart(GameStartFinish {
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
        BotEvent::GameStart(GameStartFinish {
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
        BotEvent::GameStart(GameStartFinish {
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
        BotEvent::GameStart(GameStartFinish {
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
        BotEvent::GameFinish(GameStartFinish {
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
        BotEvent::Challenge(Challenge {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: User {
                rating: None,
                provisional: false,
                online: false,
                id: "testChallengerId".to_owned(),
                name: "testChallengerName".to_owned(),
                title: None,
                patron: false
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
        BotEvent::Challenge(Challenge {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: User {
                rating: Some(2345),
                provisional: false,
                online: true,
                id: "testChallengerId".to_owned(),
                name: "testChallengerName".to_owned(),
                title: Some(Title::Wgm),
                patron: true
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
        BotEvent::Challenge(Challenge {
            id: "testId".to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: User {
                rating: None,
                provisional: false,
                online: false,
                id: "testChallengerId".to_owned(),
                name: "testChallengerName".to_owned(),
                title: None,
                patron: false
            },
            dest_user: Some(User {
                rating: None,
                provisional: false,
                online: false,
                id: "testDestUserId".to_owned(),
                name: "testDestUserName".to_owned(),
                title: None,
                patron: false
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
        BotEvent::Challenge(Challenge {
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
        BotEvent::Challenge(Challenge {
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
        BotEvent::ChallengeCanceled(Challenge {
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
        BotEvent::ChallengeCanceled(Challenge {
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
        BotEvent::ChallengeCanceled(Challenge {
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
        BotEvent::ChallengeDeclined(ChallengeDeclined {
            id: "testId".to_owned()
        })
    )]
    fn parse_bot_event(#[case] json: &str, #[case] expected_event: BotEvent) {
        let event = serde_json::from_str(json).unwrap();

        assert_that!(event).is_equal_to(expected_event);
    }
}
