use std::hash::Hash;

use serde::Deserialize;

use crate::model::{Any, Seconds, Timestamp, Url};

pub type UserId = String;
pub type Rating = i32;
pub type AiLevel = i32;

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
    // TODO rating really optional?
    pub rating: Option<Rating>,

    #[serde(default)]
    pub provisional: bool,

    #[serde(default)]
    pub online: bool,
    pub id: UserId,
    pub name: String,
    pub title: Option<Title>,

    #[serde(default)]
    pub patron: bool
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct Perf {
    pub games: u32,
    pub rating: Rating,
    pub rd: i32,
    pub prog: i32,

    #[serde(default)]
    pub prov: bool
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct PuzzleModePerf {
    pub runs: u32,
    pub score: Rating
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Perfs {
    pub chess960: Option<Perf>,
    pub atomic: Option<Perf>,
    pub racing_kings: Option<Perf>,
    pub ultra_bullet: Option<Perf>,
    pub blitz: Option<Perf>,
    pub king_of_the_hill: Option<Perf>,
    pub bullet: Option<Perf>,
    pub correspondence: Option<Perf>,
    pub horde: Option<Perf>,
    pub puzzle: Option<Perf>,
    pub classical: Option<Perf>,
    pub rapid: Option<Perf>,
    pub storm: Option<PuzzleModePerf>,
    pub racer: Option<PuzzleModePerf>,
    pub streak: Option<PuzzleModePerf>
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub country: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub fide_rating: Option<Rating>,
    pub uscf_rating: Option<Rating>,
    pub ecf_rating: Option<Rating>,
    pub links: Option<String>
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct PlayTime {
    pub total: Seconds,
    pub tv: Seconds
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileStats {
    pub all: u32,
    pub rated: u32,
    pub ai: u32,
    pub draw: u32,
    pub draw_h: u32,
    pub loss: u32,
    pub loss_h: u32,
    pub win: u32,
    pub win_h: u32,
    pub bookmark: u32,
    pub playing: u32,
    pub import: u32,
    pub me: u32
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: UserId,
    pub username: String,

    #[serde(default)]
    pub perfs: Perfs,
    pub created_at: Timestamp,

    #[serde(default)]
    pub disabled: bool,

    #[serde(default)]
    pub tos_violation: bool,

    #[serde(default)]
    pub profile: Profile,
    pub seen_at: Timestamp,

    #[serde(default)]
    pub patron: bool,

    #[serde(default)]
    pub verified: bool,
    pub play_time: PlayTime,
    pub title: Option<Title>,
    pub url: Url,
    pub playing: Option<Url>,
    pub count: UserProfileStats,

    #[serde(default)]
    pub streaming: bool,

    // TODO resolve any
    pub streamer: Option<Any>,

    #[serde(default)]
    pub followable: bool,

    #[serde(default)]
    pub following: bool,

    #[serde(default)]
    pub blocking: bool,

    #[serde(default)]
    pub follows_you: bool
}

#[cfg(test)]
mod tests {

    use kernal::prelude::*;

    use rstest::rstest;

    use crate::model::user::{
        Perf,
        Perfs,
        PlayTime,
        Profile,
        PuzzleModePerf,
        Title,
        UserProfile,
        UserProfileStats
    };

    fn minimal_user_profile() -> UserProfile {
        UserProfile {
            id: "testId".to_owned(),
            username: "testUsername".to_owned(),
            perfs: Perfs::default(),
            created_at: 123,
            disabled: false,
            tos_violation: false,
            profile: Profile::default(),
            seen_at: 321,
            patron: false,
            verified: false,
            play_time: PlayTime {
                total: 12345,
                tv: 1234
            },
            title: None,
            url: "testUrl".to_string(),
            playing: None,
            count: UserProfileStats {
                all: 1,
                rated: 2,
                ai: 3,
                draw: 4,
                draw_h: 5,
                loss: 6,
                loss_h: 7,
                win: 8,
                win_h: 9,
                bookmark: 10,
                playing: 11,
                import: 12,
                me: 13
            },
            streaming: false,
            streamer: None,
            followable: false,
            following: false,
            blocking: false,
            follows_you: false,
        }
    }

    #[rstest]
    #[case::minimal(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "createdAt": 123,
            "seenAt": 321,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            }
        }"#,
        minimal_user_profile()
    )]
    #[case::with_perf_chess960(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "perfs": {
                "chess960": {
                    "games": 12,
                    "rating": 23,
                    "rd": 34,
                    "prog": 45
                }
            },
            "createdAt": 123,
            "seenAt": 321,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            }
        }"#,
        UserProfile {
            perfs: Perfs {
                chess960: Some(Perf {
                    games: 12,
                    rating: 23,
                    rd: 34,
                    prog: 45,
                    prov: false
                }),
                ..Perfs::default()
            },
            ..minimal_user_profile()
        }
    )]
    #[case::with_perfs_other_variants(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "perfs": {
                "atomic": {
                    "games": 123,
                    "rating": 234,
                    "rd": 345,
                    "prog": 456,
                    "prov": true
                },
                "racingKings": {
                    "games": 321,
                    "rating": 432,
                    "rd": 543,
                    "prog": 654
                },
                "kingOfTheHill": {
                    "games": 1234,
                    "rating": 2345,
                    "rd": 3456,
                    "prog": 4567,
                    "prov": false
                },
                "horde": {
                    "games": 4321,
                    "rating": 5432,
                    "rd": 6543,
                    "prog": 7654,
                    "prov": true
                }
            },
            "createdAt": 123,
            "seenAt": 321,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            }
        }"#,
        UserProfile {
            perfs: Perfs {
                atomic: Some(Perf {
                    games: 123,
                    rating: 234,
                    rd: 345,
                    prog: 456,
                    prov: true
                }),
                racing_kings: Some(Perf {
                    games: 321,
                    rating: 432,
                    rd: 543,
                    prog: 654,
                    prov: false
                }),
                king_of_the_hill: Some(Perf {
                    games: 1234,
                    rating: 2345,
                    rd: 3456,
                    prog: 4567,
                    prov: false
                }),
                horde: Some(Perf {
                    games: 4321,
                    rating: 5432,
                    rd: 6543,
                    prog: 7654,
                    prov: true
                }),
                ..Perfs::default()
            },
            ..minimal_user_profile()
        }
    )]
    #[case::with_perfs_time_controls(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "perfs": {
                "ultraBullet": {
                    "games": 23,
                    "rating": 34,
                    "rd": 45,
                    "prog": 56
                },
                "bullet": {
                    "games": 34,
                    "rating": 45,
                    "rd": 56,
                    "prog": 67,
                    "prov": false
                },
                "blitz": {
                    "games": 45,
                    "rating": 56,
                    "rd": 67,
                    "prog": 78,
                    "prov": true
                },
                "rapid": {
                    "games": 56,
                    "rating": 67,
                    "rd": 78,
                    "prog": 89,
                    "prov": true
                },
                "classical": {
                    "games": 67,
                    "rating": 78,
                    "rd": 89,
                    "prog": 90
                },
                "correspondence": {
                    "games": 78,
                    "rating": 89,
                    "rd": 90,
                    "prog": 1,
                    "prov": true
                }
            },
            "createdAt": 123,
            "seenAt": 321,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            }
        }"#,
        UserProfile {
            perfs: Perfs {
                ultra_bullet: Some(Perf {
                    games: 23,
                    rating: 34,
                    rd: 45,
                    prog: 56,
                    prov: false
                }),
                bullet: Some(Perf {
                    games: 34,
                    rating: 45,
                    rd: 56,
                    prog: 67,
                    prov: false
                }),
                blitz: Some(Perf {
                    games: 45,
                    rating: 56,
                    rd: 67,
                    prog: 78,
                    prov: true
                }),
                rapid: Some(Perf {
                    games: 56,
                    rating: 67,
                    rd: 78,
                    prog: 89,
                    prov: true
                }),
                classical: Some(Perf {
                    games: 67,
                    rating: 78,
                    rd: 89,
                    prog: 90,
                    prov: false
                }),
                correspondence: Some(Perf {
                    games: 78,
                    rating: 89,
                    rd: 90,
                    prog: 1,
                    prov: true
                }),
                ..Perfs::default()
            },
            ..minimal_user_profile()
        }
    )]
    #[case::with_perfs_puzzle_modes(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "perfs": {
                "puzzle": {
                    "games": 100,
                    "rating": 200,
                    "rd": 300,
                    "prog": 400
                },
                "storm": {
                    "runs": 10,
                    "score": 20
                },
                "racer": {
                    "runs": 30,
                    "score": 40
                },
                "streak": {
                    "runs": 50,
                    "score": 60
                }
            },
            "createdAt": 123,
            "seenAt": 321,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            }
        }"#,
        UserProfile {
            perfs: Perfs {
                puzzle: Some(Perf {
                    games: 100,
                    rating: 200,
                    rd: 300,
                    prog: 400,
                    prov: false
                }),
                storm: Some(PuzzleModePerf {
                    runs: 10,
                    score: 20
                }),
                racer: Some(PuzzleModePerf {
                    runs: 30,
                    score: 40
                }),
                streak: Some(PuzzleModePerf {
                    runs: 50,
                    score: 60
                }),
                ..Perfs::default()
            },
            ..minimal_user_profile()
        }
    )]
    #[case::with_empty_profile(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "createdAt": 123,
            "profile": { },
            "seenAt": 321,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            }
        }"#,
        minimal_user_profile()
    )]
    #[case::with_partial_profile(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "createdAt": 123,
            "profile": {
                "firstName": "Max",
                "lastName": "Mustermann"
            },
            "seenAt": 321,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            }
        }"#,
        UserProfile {
            profile: Profile {
                country: None,
                location: None,
                bio: None,
                first_name: Some("Max".to_owned()),
                last_name: Some("Mustermann".to_owned()),
                fide_rating: None,
                uscf_rating: None,
                ecf_rating: None,
                links: None
            },
            ..minimal_user_profile()
        }
    )]
    #[case::with_full_profile(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "createdAt": 123,
            "profile": {
                "country": "Testonia",
                "location": "Testockholm",
                "bio": "I am for tests",
                "firstName": "Max",
                "lastName": "Mustermann",
                "fideRating": 234,
                "uscfRating": 345,
                "ecfRating": 456,
                "links": "links.test"
            },
            "seenAt": 321,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            }
        }"#,
        UserProfile {
            profile: Profile {
                country: Some("Testonia".to_owned()),
                location: Some("Testockholm".to_owned()),
                bio: Some("I am for tests".to_owned()),
                first_name: Some("Max".to_owned()),
                last_name: Some("Mustermann".to_owned()),
                fide_rating: Some(234),
                uscf_rating: Some(345),
                ecf_rating: Some(456),
                links: Some("links.test".to_owned())
            },
            ..minimal_user_profile()
        }
    )]
    #[case::with_title(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "createdAt": 123,
            "seenAt": 321,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "title": "FM",
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            }
        }"#,
        UserProfile {
            title: Some(Title::Fm),
            ..minimal_user_profile()
        }
    )]
    #[case::with_playing_url(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "createdAt": 123,
            "seenAt": 321,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "playing": "testPlayingUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            }
        }"#,
        UserProfile {
            playing: Some("testPlayingUrl".to_owned()),
            ..minimal_user_profile()
        }
    )]
    #[case::with_some_flags(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "createdAt": 123,
            "disabled": true,
            "seenAt": 321,
            "patron": true,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            },
            "streaming": true,
            "following": true,
            "followsYou": true
        }"#,
        UserProfile {
            disabled: true,
            patron: true,
            streaming: true,
            following: true,
            follows_you: true,
            ..minimal_user_profile()
        }
    )]
    #[case::with_other_flags(
        r#"{
            "id": "testId",
            "username": "testUsername",
            "createdAt": 123,
            "tosViolation": true,
            "seenAt": 321,
            "verified": true,
            "playTime": {
                "total": 12345,
                "tv": 1234
            },
            "url": "testUrl",
            "count": {
                "all": 1,
                "rated": 2,
                "ai": 3,
                "draw": 4,
                "drawH": 5,
                "loss": 6,
                "lossH": 7,
                "win": 8,
                "winH": 9,
                "bookmark": 10,
                "playing": 11,
                "import": 12,
                "me": 13
            },
            "followable": true,
            "blocking": true
        }"#,
        UserProfile {
            tos_violation: true,
            verified: true,
            followable: true,
            blocking: true,
            ..minimal_user_profile()
        }
    )]
    fn deserialize_user_profile(#[case] json: &str, #[case] expected_profile: UserProfile) {
        let user_profile = serde_json::from_str(json).unwrap();

        assert_that!(user_profile).is_equal_to(expected_profile);
    }
}
