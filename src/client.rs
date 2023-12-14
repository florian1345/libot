use std::sync::Arc;

use crate::error::{BotClientBuilderError, BotClientBuilderResult, LibotRequestError, LibotResult};
use crate::model::{ChatRoom, DeclineReason, DeclineRequest, GameId, Move, Seconds, SendChatMessageRequest, UserProfile};

use reqwest::{Client, ClientBuilder, Method, Response};
use reqwest::Result as ReqwestResult;
use reqwest::header::{AUTHORIZATION, HeaderMap};

use serde::Serialize;

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
        return Err(LibotRequestError::ApiError {
            status: response.status(),
            body: response.text().await.ok()
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

    pub async fn get_my_profile(&self) -> LibotResult<UserProfile> {
        Ok(self.send_request(Method::GET, "/account").await?.json().await?)
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

    use super::*;

    use kernal::prelude::*;
    use rstest::rstest;
    use wiremock::{Mock, ResponseTemplate};
    use wiremock::matchers::{body_json_string, body_string, method, path, query_param};
    use crate::test_util;

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
}
