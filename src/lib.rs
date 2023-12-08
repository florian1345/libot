use std::fmt::Debug;

use futures::Stream;
use futures::stream::StreamExt;

use ndjson_stream::config::{EmptyLineHandling, NdjsonConfig};

use reqwest::{Client, ClientBuilder, Error as ReqwestError, Method, Response, Result as ReqwestResult};
use reqwest::header::{AUTHORIZATION, HeaderMap, InvalidHeaderValue};

use thiserror::Error;

use crate::model::{Challenge, ChallengeDeclined, Event, GameEventInfo};

mod model;

#[derive(Debug)]
pub struct BotClient {
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

impl BotClient {
    pub(crate) async fn send_request(&self, method: Method, path: &str) -> ReqwestResult<Response> {
        let url = join_url(&self.base_url, path);
        self.client.request(method, url).send().await
    }
}

const DEFAULT_BASE_URL: &str = "https://lichess.org/api";

#[derive(Debug, Error)]
pub enum BotClientBuilderError {
    #[error("no token specified")]
    NoToken,

    #[error("token is invalid: {0}")]
    InvalidToken(#[from] InvalidHeaderValue),

    #[error("error initializing client: {0}")]
    ClientError(#[from] ReqwestError)
}

pub type BotClientBuilderResult = Result<BotClient, BotClientBuilderError>;

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

#[async_trait::async_trait]
pub trait Bot {

    async fn on_game_start(&self, game: GameEventInfo);

    async fn on_game_finish(&self, game: GameEventInfo);

    async fn on_challenge(&self, challenge: Challenge);

    async fn on_challenge_cancelled(&self, challenge: Challenge);

    async fn on_challenge_declined(&self, challenge: ChallengeDeclined);
}

const EVENT_PATH: &str = "/stream/event";

async fn run_with_event_stream<E>(bot: impl Bot, event_stream: impl Stream<Item = Result<Event, E>>)
where
    E: Debug
{
    event_stream.for_each(|record| async {
        // TODO enable error handling
        match record.unwrap() {
            Event::GameStart(game) => bot.on_game_start(game).await,
            Event::GameFinish(game) => bot.on_game_finish(game).await,
            Event::Challenge(challenge) => bot.on_challenge(challenge).await,
            Event::ChallengeCanceled(challenge) => bot.on_challenge_cancelled(challenge).await,
            Event::ChallengeDeclined(challenge) => bot.on_challenge_declined(challenge).await
        }
    }).await
}

pub async fn run(bot: impl Bot, client: BotClient) -> reqwest::Result<()> {
    let response = client.send_request(Method::GET, EVENT_PATH).await?;
    let ndjson_config = NdjsonConfig::default()
        .with_empty_line_handling(EmptyLineHandling::IgnoreEmpty);
    let stream =
        ndjson_stream::from_fallible_stream_with_config::<Event, _>(
            response.bytes_stream(), ndjson_config);

    #[allow(clippy::unit_arg)]
    Ok(run_with_event_stream(bot, stream).await)
}

#[cfg(test)]
mod tests {
    use kernal::prelude::*;

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
}
