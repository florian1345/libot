use crate::error::{BotClientBuilderError, BotClientBuilderResult, LibotResult};
use crate::model::{DeclineReason, DeclineRequest, GameId};

use reqwest::{Client, ClientBuilder, Method, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap};

use serde::Serialize;

#[derive(Debug)]
pub struct BotClient {
    client: Client,
    base_url: String
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

impl BotClient {
    pub(crate) async fn send_request(&self, method: Method, path: &str)
            -> LibotResult<Response> {
        let url = join_url(&self.base_url, path);

        Ok(self.client.request(method, url).send().await?)
    }

    pub(crate) async fn send_request_with_body(&self, method: Method, path: &str,
            body: impl Serialize) -> LibotResult<Response> {
        let url = join_url(&self.base_url, path);
        let body = serde_json::to_string(&body)?;

        Ok(self.client.request(method, url).body(body).send().await?)
    }

    pub async fn accept_challenge(&self, challenge_id: GameId) -> LibotResult<()> {
        // TODO error handling
        let path = format!("/challenge/{challenge_id}/accept");
        self.send_request(Method::POST, &path).await?;

        Ok(())
    }

    pub async fn decline_challenge(&self, challenge_id: GameId, reason: Option<DeclineReason>)
            -> LibotResult<()> {
        // TODO error handling
        let path = format!("/challenge/{challenge_id}/decline");
        let body = DeclineRequest {
            reason
        };
        self.send_request_with_body(Method::POST, &path, body).await?;

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
                base_url: self.base_url
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
    use wiremock::{Mock, ResponseTemplate};
    use wiremock::matchers::{body_json_string, method, path};
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
        assert_that!(result.unwrap().base_url.as_str()).is_equal_to(DEFAULT_BASE_URL);
    }

    #[test]
    fn building_bot_client_succeeds_with_valid_token_and_overridden_base_url() {
        let base_url = "https://base.url/path";
        let result = BotClientBuilder::new()
            .with_token("abc123")
            .with_base_url(base_url)
            .build();

        assert_that!(&result).is_ok();
        assert_that!(result.unwrap().base_url.as_str()).is_equal_to(base_url);
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
}
