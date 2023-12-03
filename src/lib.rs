use reqwest::header::{AUTHORIZATION, HeaderMap, InvalidHeaderValue};
use reqwest::{Client, ClientBuilder, Error as ReqwestError, Method, Response, Result as ReqwestResult};

use thiserror::Error;

#[derive(Debug)]
pub struct Bot {
    client: Client,
    base_url: String
}

fn join_url(base_url: &str, path: &str) -> String {
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

impl Bot {
    pub async fn send_request(&mut self, method: Method, path: &str) -> ReqwestResult<Response> {
        let url = join_url(&self.base_url, path);
        self.client.request(method, url).send().await
    }
}

const DEFAULT_BASE_URL: &str = "https://lichess.org/api";

#[derive(Debug, Error)]
pub enum BotBuilderError {
    #[error("no token specified")]
    NoToken,

    #[error("token is invalid: {0}")]
    InvalidToken(#[from] InvalidHeaderValue),

    #[error("error initializing client: {0}")]
    ClientError(#[from] ReqwestError)
}

pub type BotBuilderResult = Result<Bot, BotBuilderError>;

pub struct BotBuilder {
    token: Option<String>,
    base_url: String
}

impl BotBuilder {

    pub fn new() -> BotBuilder {
        BotBuilder {
            token: None,
            base_url: DEFAULT_BASE_URL.to_owned()
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> BotBuilder {
        self.token = Some(token.into());
        self
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> BotBuilder {
        self.base_url = base_url.into();
        self
    }

    pub fn build(self) -> BotBuilderResult {
        if let Some(token) = self.token {
            let mut headers = HeaderMap::new();
            let authorization_value = format!("Bearer {}", token).parse()?;
            headers.insert(AUTHORIZATION, authorization_value);
            let client = ClientBuilder::new().default_headers(headers).build()?;

            Ok(Bot {
                client,
                base_url: self.base_url
            })
        }
        else {
            Err(BotBuilderError::NoToken)
        }
    }
}

impl Default for BotBuilder {
    fn default() -> BotBuilder {
        BotBuilder::new()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use kernal::prelude::*;

    #[test]
    fn building_bot_fails_without_token() {
        let result = BotBuilder::new().build();

        assert!(matches!(result, Err(BotBuilderError::NoToken)));
    }

    #[test]
    fn building_bot_fails_with_invalid_token() {
        let result = BotBuilder::new()
            .with_token("\0")
            .build();

        assert!(matches!(result, Err(BotBuilderError::InvalidToken(_))));
    }

    #[test]
    fn building_bot_succeeds_with_valid_token_and_default_base_url() {
        let result = BotBuilder::new()
            .with_token("abc123")
            .build();

        assert_that!(&result).is_ok();
        assert_that!(result.unwrap().base_url.as_str()).is_equal_to(DEFAULT_BASE_URL);
    }

    #[test]
    fn building_bot_succeeds_with_valid_token_and_overridden_base_url() {
        let base_url = "https://base.url/path";
        let result = BotBuilder::new()
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
}
