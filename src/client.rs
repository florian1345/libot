use std::sync::Arc;

use reqwest::{Client, ClientBuilder, Method, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap};
use reqwest::Result as ReqwestResult;

use serde::Serialize;

use crate::error::{BotClientBuilderError, BotClientBuilderResult, LibotRequestError, LibotResult};
use crate::model::{Move, Seconds};
use crate::model::challenge::DeclineReason;
use crate::model::game::chat::{ChatHistory, ChatRoom};
use crate::model::game::GameId;
use crate::model::request::{DeclineRequest, SendChatMessageRequest};
use crate::model::user::preferences::UserPreferences;
use crate::model::user::UserProfile;

#[derive(Clone, Debug)]
pub struct BotClient {
    client: Client,
    base_url: Arc<str>
}

pub fn join_url(base_url: &str, path: &str) -> String {
    let mut url = base_url.to_owned();

    if url.ends_with('/') {
        url.pop();
    }

    if !path.starts_with('/') {
        url.push('/');
    }

    url.push_str(path);
    url
}

async fn handle_error(response: ReqwestResult<Response>) -> LibotResult<Response> {
    let response = response?;

    if !response.status().is_success() {
        let status = response.status();
        let url = response.url().clone();

        return Err(LibotRequestError::ApiError {
            status,
            body: response.text().await.ok(),
            url
        });
    }

    Ok(response)
}

impl BotClient {

    pub(crate) async fn send_request(&self, method: Method, path: &str)
            -> LibotResult<Response> {
        let url = join_url(&self.base_url, path);

        handle_error(self.client.request(method, url).send().await).await
    }

    pub(crate) async fn send_request_with_body(&self, method: Method, path: &str,
            body: impl Serialize) -> LibotResult<Response> {
        let url = join_url(&self.base_url, path);

        handle_error(self.client.request(method, url).json(&body).send().await).await
    }

    pub(crate) async fn send_request_with_form(&self, method: Method, path: &str,
            form: impl Serialize) -> LibotResult<Response> {
        let url = join_url(&self.base_url, path);

        handle_error(self.client.request(method, url).form(&form).send().await).await
    }

    pub(crate) async fn send_request_with_query(&self, method: Method, path: &str,
            query: impl Serialize) -> LibotResult<Response> {
        let url = join_url(&self.base_url, path);

        handle_error(self.client.request(method, url).query(&query).send().await).await
    }

    pub async fn accept_challenge(&self, challenge_id: GameId) -> LibotResult<()> {
        let path = format!("/challenge/{challenge_id}/accept");
        self.send_request(Method::POST, &path).await?;

        Ok(())
    }

    pub async fn decline_challenge(&self, challenge_id: GameId, reason: Option<DeclineReason>)
            -> LibotResult<()> {
        let path = format!("/challenge/{challenge_id}/decline");
        let body = DeclineRequest {
            reason
        };
        self.send_request_with_body(Method::POST, &path, body).await?;

        Ok(())
    }

    pub async fn make_move(&self, game_id: GameId, mov: Move, offer_draw: bool) -> LibotResult<()> {
        #[derive(Serialize)]
        struct OfferDraw {
            #[serde(rename = "offeringDraw")]
            offer_draw: bool
        }

        let path = format!("/bot/game/{game_id}/move/{mov}");
        let query = OfferDraw { offer_draw };

        self.send_request_with_query(Method::POST, &path, query).await?;

        Ok(())
    }

    pub async fn get_game_chat(&self, game_id: GameId) -> LibotResult<ChatHistory> {
        let path = format!("/bot/game/{game_id}/chat");

        Ok(self.send_request(Method::GET, &path).await?.json().await?)
    }

    pub async fn send_chat_message(&self, game_id: GameId, room: ChatRoom, text: impl Into<String>)
            -> LibotResult<()> {
        let path = format!("/bot/game/{game_id}/chat");
        let body = SendChatMessageRequest {
            room,
            text: text.into()
        };

        self.send_request_with_form(Method::POST, &path, body).await?;

        Ok(())
    }

    /// Aborts a game which is currently being played and in which this bot is participating.
    ///
    /// # Arguments
    ///
    /// * `game_id`: The ID of the game to resign.
    pub async fn abort_game(&self, game_id: GameId) -> LibotResult<()> {
        let path = format!("/bot/game/{game_id}/abort");

        self.send_request(Method::POST, &path).await?;

        Ok(())
    }

    /// Resign a game which is currently being played and in which this bot is participating.
    ///
    /// # Arguments
    ///
    /// * `game_id`: The ID of the game to resign.
    pub async fn resign_game(&self, game_id: GameId) -> LibotResult<()> {
        let path = format!("/bot/game/{game_id}/resign");

        self.send_request(Method::POST, &path).await?;

        Ok(())
    }

    /// Queries the [UserProfile] of the user with the given name.
    ///
    /// # Arguments
    ///
    /// * `username`: The username of the user whose profile to query.
    pub async fn get_profile(&self, username: String) -> LibotResult<UserProfile> {
        let path = format!("/user/{username}");

        Ok(self.send_request(Method::GET, &path).await?.json().await?)
    }

    pub async fn get_my_profile(&self) -> LibotResult<UserProfile> {
        Ok(self.send_request(Method::GET, "/account").await?.json().await?)
    }

    pub async fn get_my_preferences(&self) -> LibotResult<UserPreferences> {
        Ok(self.send_request(Method::GET, "/account/preferences").await?.json().await?)
    }

    /// Adds time to the opponent's clock.
    ///
    /// # Arguments
    ///
    /// * `game_id`: ID of the game in which to give time to the bot's opponent.
    /// * `seconds`: The number of seconds to give the bot's opponent.
    pub async fn add_time(&self, game_id: GameId, seconds: Seconds) -> LibotResult<()> {
        let path = format!("/round/{game_id}/add-time/{seconds}");
        self.send_request(Method::POST, &path).await?;

        Ok(())
    }
}

pub const DEFAULT_BASE_URL: &str = "https://lichess.org/api";

pub struct BotClientBuilder {
    token: Option<String>,
    base_url: String
}

impl BotClientBuilder {

    pub fn new() -> BotClientBuilder {
        BotClientBuilder {
            token: None,
            base_url: DEFAULT_BASE_URL.to_owned()
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> BotClientBuilder {
        self.token = Some(token.into());
        self
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> BotClientBuilder {
        self.base_url = base_url.into();
        self
    }

    pub fn build(self) -> BotClientBuilderResult {
        if let Some(token) = self.token {
            let mut headers = HeaderMap::new();
            let authorization_value = format!("Bearer {}", token).parse()?;
            headers.insert(AUTHORIZATION, authorization_value);
            let client = ClientBuilder::new().default_headers(headers).build()?;

            Ok(BotClient {
                client,
                base_url: Arc::from(self.base_url)
            })
        }
        else {
            Err(BotClientBuilderError::NoToken)
        }
    }
}

impl Default for BotClientBuilder {
    fn default() -> BotClientBuilder {
        BotClientBuilder::new()
    }
}

#[cfg(test)]
mod tests {

    use kernal::prelude::*;

    use rstest::rstest;

    use wiremock::{Mock, ResponseTemplate};
    use wiremock::matchers::{body_json_string, body_string, method, path, query_param};

    use crate::model::game::chat::ChatLine;
    use crate::model::user::{PlayTime, UserProfileStats};
    use crate::model::user::preferences::{
        AutoQueen,
        AutoThreefold,
        CastlingMethod,
        ChallengeFilter,
        ClockTenths,
        Coordinates,
        InsightShare,
        MessageFilter,
        MoreTime,
        MoveConfirmations,
        MoveEvent,
        PieceAnimation,
        Replay,
        TakeBack,
        ZenMode
    };
    use crate::test_util;

    use super::*;

    #[test]
    fn building_bot_client_fails_without_token() {
        let result = BotClientBuilder::new().build();

        assert!(matches!(result, Err(BotClientBuilderError::NoToken)));
    }

    #[test]
    fn building_bot_client_fails_with_invalid_token() {
        let result = BotClientBuilder::new()
            .with_token("\0")
            .build();

        assert!(matches!(result, Err(BotClientBuilderError::InvalidToken(_))));
    }

    #[test]
    fn building_bot_client_succeeds_with_valid_token_and_default_base_url() {
        let result = BotClientBuilder::new()
            .with_token("abc123")
            .build();

        assert_that!(&result).is_ok();
        assert_that!(result.unwrap().base_url.as_ref()).is_equal_to(DEFAULT_BASE_URL);
    }

    #[test]
    fn building_bot_client_succeeds_with_valid_token_and_overridden_base_url() {
        let base_url = "https://base.url/path";
        let result = BotClientBuilder::new()
            .with_token("abc123")
            .with_base_url(base_url)
            .build();

        assert_that!(&result).is_ok();
        assert_that!(result.unwrap().base_url.as_ref()).is_equal_to(base_url);
    }

    #[test]
    fn joining_url_works_if_no_slash_is_present() {
        let base_url = "https://base.url/path";
        let path = "sub/path";

        let url = join_url(base_url, path);

        assert_that!(url.as_str()).is_equal_to("https://base.url/path/sub/path");
    }

    #[test]
    fn joining_url_works_if_base_url_has_slash() {
        let base_url = "https://lichess.org/";
        let path = "my/path";

        let url = join_url(base_url, path);

        assert_that!(url.as_str()).is_equal_to("https://lichess.org/my/path");
    }

    #[test]
    fn joining_url_works_if_base_path_has_slash() {
        let base_url = "https://lichess.org/api";
        let path = "/sub/path";

        let url = join_url(base_url, path);

        assert_that!(url.as_str()).is_equal_to("https://lichess.org/api/sub/path");
    }

    #[test]
    fn joining_url_works_if_both_have_slash() {
        let base_url = "https://lichess.org/api/";
        let path = "/bot/whatever";

        let url = join_url(base_url, path);

        assert_that!(url.as_str()).is_equal_to("https://lichess.org/api/bot/whatever");
    }

    #[test]
    fn accept_challenge_success() {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("POST"))
                .and(path("/challenge/testChallengeId/accept"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&server)
                .await;

            let result = client.accept_challenge("testChallengeId".to_owned()).await;

            assert_that!(result).is_ok();
        });
    }

    #[test]
    fn decline_challenge_success_without_reason() {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("POST"))
                .and(path("/challenge/testChallengeId/decline"))
                .and(body_json_string("{}"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&server)
                .await;

            let result = client.decline_challenge("testChallengeId".to_owned(), None).await;

            assert_that!(result).is_ok();
        });
    }

    #[test]
    fn decline_challenge_success_with_reason() {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("POST"))
                .and(path("/challenge/testChallengeId/decline"))
                .and(body_json_string("{\"reason\":\"generic\"}"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&server)
                .await;

            let result = client.decline_challenge(
                "testChallengeId".to_owned(), Some(DeclineReason::Generic)).await;

            assert_that!(result).is_ok();
        });
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn make_move(#[case] offer_draw: bool) {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("POST"))
                .and(path("/bot/game/testGameId/move/testMove"))
                .and(query_param("offeringDraw", offer_draw.to_string()))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&server)
                .await;

            let result =
                client.make_move("testGameId".to_owned(), "testMove".to_owned(), offer_draw).await;

            assert_that!(result).is_ok();
        });
    }

    #[rstest]
    #[case::empty("[]", vec![])]
    #[case::single_entry(
        r#"[
            {
                "username": "testUsername",
                "text": "testText"
            }
        ]"#,
        vec![
            ChatLine {
                username: "testUsername".to_owned(),
                text: "testText".to_owned()
            }
        ]
    )]
    #[case::multiple_entries(
        r#"[
            {
                "username": "testUsername1",
                "text": "testText1"
            },
            {
                "username": "testUsername2",
                "text": "testText2"
            }
        ]"#,
        vec![
            ChatLine {
                username: "testUsername1".to_owned(),
                text: "testText1".to_owned()
            },
            ChatLine {
                username: "testUsername2".to_owned(),
                text: "testText2".to_owned()
            }
        ]
    )]
    fn get_game_chat(#[case] json: &str, #[case] expected_chat_history: ChatHistory) {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("GET"))
                .and(path("/bot/game/testGameId/chat"))
                .respond_with(ResponseTemplate::new(200)
                    .set_body_string(json))
                .expect(1)
                .mount(&server)
                .await;

            let result = client.get_game_chat("testGameId".to_owned()).await;

            assert_that!(result).contains_value(expected_chat_history);
        });
    }

    #[test]
    fn send_chat_message() {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("POST"))
                .and(path("/bot/game/testGameId/chat"))
                .and(body_string("room=player&text=testText"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&server)
                .await;

            let result = client
                .send_chat_message("testGameId".to_owned(), ChatRoom::Player, "testText").await;

            assert_that!(result).is_ok();
        });
    }

    #[test]
    fn abort_game() {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("POST"))
                .and(path("/bot/game/testGameId/abort"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&server)
                .await;

            let result = client.abort_game("testGameId".to_owned()).await;

            assert_that!(result).is_ok();
        });
    }

    #[test]
    fn resign_game() {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("POST"))
                .and(path("/bot/game/testGameId/resign"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&server)
                .await;

            let result = client.resign_game("testGameId".to_owned()).await;

            assert_that!(result).is_ok();
        });
    }

    #[test]
    fn add_time() {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("POST"))
                .and(path("/round/testGameId/add-time/240"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&server)
                .await;

            let result = client.add_time("testGameId".to_owned(), 240).await;

            assert_that!(result).is_ok();
        });
    }

    fn get_test_user_json() -> &'static str {
        r#"{
            "id": "testId",
            "username": "testName",
            "createdAt": 12345,
            "seenAt": 23456,
            "playTime": {
                "total": 34567,
                "tv": 4567
            },
            "url": "testUrl",
            "count": {
                "all": 123,
                "rated": 234,
                "ai": 345,
                "draw": 456,
                "drawH": 567,
                "loss": 678,
                "lossH": 789,
                "win": 890,
                "winH": 123,
                "bookmark": 234,
                "playing": 345,
                "import": 456,
                "me": 567
            }
        }"#
    }

    fn get_test_user() -> UserProfile {
        UserProfile {
            id: "testId".to_string(),
            username: "testName".to_string(),
            perfs: Default::default(),
            created_at: 12345,
            disabled: false,
            tos_violation: false,
            profile: Default::default(),
            seen_at: 23456,
            patron: false,
            verified: false,
            play_time: PlayTime {
                total: 34567,
                tv: 4567
            },
            title: None,
            url: "testUrl".to_string(),
            playing: None,
            count: UserProfileStats {
                all: 123,
                rated: 234,
                ai: 345,
                draw: 456,
                draw_h: 567,
                loss: 678,
                loss_h: 789,
                win: 890,
                win_h: 123,
                bookmark: 234,
                playing: 345,
                import: 456,
                me: 567
            },
            streaming: false,
            streamer: None,
            followable: false,
            following: false,
            blocking: false,
            follows_you: false,
        }
    }

    #[test]
    fn get_profile() {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("GET"))
                .and(path("/user/testId"))
                .respond_with(ResponseTemplate::new(200)
                    .set_body_string(get_test_user_json()))
                .expect(1)
                .mount(&server)
                .await;

            let result = client.get_profile("testId".to_owned()).await;

            assert_that!(result).contains_value(get_test_user());
        })
    }

    #[test]
    fn get_my_profile() {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("GET"))
                .and(path("/account"))
                .respond_with(ResponseTemplate::new(200)
                    .set_body_string(get_test_user_json()))
                .expect(1)
                .mount(&server)
                .await;

            let result = client.get_my_profile().await;

            assert_that!(result).contains_value(get_test_user());
        })
    }

    #[test]
    fn get_my_preferences() {
        tokio_test::block_on(async {
            let preferences = UserPreferences {
                dark: false,
                transparent: false,
                background_image: "testBackgroundImage".to_owned(),
                is_3d: false,
                theme: "testTheme".to_owned(),
                piece_set: "testPieceSet".to_owned(),
                theme_3d: "testTheme3d".to_owned(),
                piece_set_3d: "testPieceSet3d".to_owned(),
                sound_set: "testSoundSet".to_owned(),
                blindfold: false,
                auto_queen: AutoQueen::Never,
                auto_threefold: AutoThreefold::WhenLessThan30Seconds,
                take_back: TakeBack::Never,
                more_time: MoreTime::Never,
                clock_tenths: ClockTenths::Always,
                clock_bar: false,
                clock_sound: false,
                premove: false,
                animation: PieceAnimation::None,
                captured: false,
                follow: false,
                highlight: false,
                destination: false,
                coords: Coordinates::Inside,
                replay: Replay::Always,
                challenge: ChallengeFilter::OnlyFriends,
                message: MessageFilter::OnlyExistingConversations,
                move_confirmations: MoveConfirmations::EMPTY,
                confirm_resign: true,
                insight_share: InsightShare::WithEverybody,
                keyboard_move: false,
                zen: ZenMode::Yes,
                ratings: true,
                move_event: MoveEvent::Either,
                castling_method: CastlingMethod::KingTwoSquares,
                language: "testLanguage".to_owned()
            };
            let preferences_json = r#"{
                "prefs": {
                    "bgImg": "testBackgroundImage",
                    "theme": "testTheme",
                    "pieceSet": "testPieceSet",
                    "theme3d": "testTheme3d",
                    "pieceSet3d": "testPieceSet3d",
                    "soundSet": "testSoundSet",
                    "blindfold": 0,
                    "autoQueen": 1,
                    "autoThreefold": 2,
                    "takeback": 1,
                    "moretime": 1,
                    "clockTenths": 2,
                    "animation": 0,
                    "coords": 1,
                    "replay": 2,
                    "challenge": 3,
                    "message": 1,
                    "submitMove": 0,
                    "confirmResign": 1,
                    "insightShare": 2,
                    "keyboardMove": 0,
                    "zen": 1,
                    "ratings": 1,
                    "moveEvent": 2,
                    "rookCastle": 0
                },
                "language": "testLanguage"
            }"#;

            let (client, server) = test_util::setup_wiremock_test().await;

            Mock::given(method("GET"))
                .and(path("/account/preferences"))
                .respond_with(ResponseTemplate::new(200)
                    .set_body_string(preferences_json))
                .expect(1)
                .mount(&server)
                .await;

            let result = client.get_my_preferences().await;

            assert_that!(result).contains_value(preferences);
        })
    }
}
