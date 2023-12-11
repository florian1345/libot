use std::fmt::Debug;

use futures::Stream;
use futures::stream::StreamExt;

use ndjson_stream::config::{EmptyLineHandling, NdjsonConfig};

use reqwest::Method;

use crate::client::BotClient;
use crate::error::LibotResult;
use crate::model::{ChallengeEvent, ChallengeDeclinedEvent, BotEvent, GameStartFinishEvent, GameId, GameFullEvent, GameStateEvent, ChatLineEvent, OpponentGoneEvent, GameEvent};

pub mod model;
pub mod error;
pub mod client;

#[cfg(test)]
pub(crate) mod test_util;

#[async_trait::async_trait]
pub trait Bot : Sync {

    async fn on_game_start(&self, _game: GameStartFinishEvent, _client: &BotClient) { }

    async fn on_game_finish(&self, _game: GameStartFinishEvent, _client: &BotClient) { }

    async fn on_challenge(&self, _challenge: ChallengeEvent, _client: &BotClient) { }

    async fn on_challenge_cancelled(&self, _challenge: ChallengeEvent, _client: &BotClient) { }

    async fn on_challenge_declined(&self, _challenge: ChallengeDeclinedEvent, _client: &BotClient)
        { }

    async fn on_game_full(&self, _id: GameId, _game_full: GameFullEvent, _client: &BotClient) { }

    async fn on_game_state(&self, _id: GameId, _state: GameStateEvent, _client: &BotClient) { }

    async fn on_chat_line(&self, _id: GameId, _chat_line: ChatLineEvent, _client: &BotClient) { }

    async fn on_opponent_gone(&self, _id: GameId, _opponent_gone: OpponentGoneEvent,
        _client: &BotClient) { }
}

const EVENT_PATH: &str = "/stream/event";

fn game_event_path(game_id: &GameId) -> String {
    format!("/bot/game/stream/{}", game_id)
}

async fn run_with_game_event_stream<E>(bot: &impl Bot,
    event_stream: impl Stream<Item = Result<GameEvent, E>>, client: &BotClient, game_id: GameId)
where
    E: Debug
{
    event_stream.for_each(|record| async {
        // TODO enable error handling
        match record.unwrap() {
            GameEvent::GameFull(game_full) =>
                bot.on_game_full(game_id.clone(), game_full, client).await,
            GameEvent::GameState(state) =>
                bot.on_game_state(game_id.clone(), state, client).await,
            GameEvent::ChatLine(chat_line) =>
                bot.on_chat_line(game_id.clone(), chat_line, client).await,
            GameEvent::OpponentGone(opponent_gone) =>
                bot.on_opponent_gone(game_id.clone(), opponent_gone, client).await,
        }
    }).await
}

async fn run_with_event_stream<E>(bot: impl Bot,
    event_stream: impl Stream<Item = Result<BotEvent, E>>, client: &BotClient)
where
    E: Debug
{
    event_stream.for_each_concurrent(None, |record| async {
        // TODO enable error handling
        match record.unwrap() {
            BotEvent::GameStart(game) => {
                let game_id = game.id.clone();
                bot.on_game_start(game, client).await;

                if let Some(game_id) = game_id {
                    let event_path = game_event_path(&game_id);

                    // TODO enable error handling
                    if let Ok(response) = client.send_request(Method::GET, &event_path).await {
                        let stream =
                            ndjson_stream::from_fallible_stream_with_config::<GameEvent, _>(
                                response.bytes_stream(), ndjson_config());

                        run_with_game_event_stream(&bot, stream, client, game_id).await
                    }
                }
            },
            BotEvent::GameFinish(game) => bot.on_game_finish(game, client).await,
            BotEvent::Challenge(challenge) => bot.on_challenge(challenge, client).await,
            BotEvent::ChallengeCanceled(challenge) =>
                bot.on_challenge_cancelled(challenge, client).await,
            BotEvent::ChallengeDeclined(challenge) =>
                bot.on_challenge_declined(challenge, client).await
        }
    }).await
}

pub async fn run(bot: impl Bot, client: BotClient) -> LibotResult<()> {
    let response = client.send_request(Method::GET, EVENT_PATH).await?;
    let stream =
        ndjson_stream::from_fallible_stream_with_config::<BotEvent, _>(
            response.bytes_stream(), ndjson_config());

    #[allow(clippy::unit_arg)]
    Ok(run_with_event_stream(bot, stream, &client).await)
}

fn ndjson_config() -> NdjsonConfig {
    NdjsonConfig::default()
        .with_empty_line_handling(EmptyLineHandling::IgnoreEmpty)
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use std::sync::{Arc, Mutex};

    use futures::stream;
    use kernal::prelude::*;
    use rstest::rstest;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    use crate::client::BotClientBuilder;
    use crate::model::{ChallengeColor, ChallengePerf, ChallengeStatus, ChatRoom, Speed, TimeControl, User};

    use super::*;

    struct MockBot {
        bot_events: Arc<Mutex<Vec<BotEvent>>>,
        game_events: Arc<Mutex<Vec<GameEvent>>>
    }

    #[async_trait::async_trait]
    impl Bot for MockBot {
        async fn on_game_start(&self, game: GameStartFinishEvent, _: &BotClient) {
            self.bot_events.lock().unwrap().push(BotEvent::GameStart(game));
        }

        async fn on_game_finish(&self, game: GameStartFinishEvent, _: &BotClient) {
            self.bot_events.lock().unwrap().push(BotEvent::GameFinish(game));
        }

        async fn on_challenge(&self, challenge: ChallengeEvent, _: &BotClient) {
            self.bot_events.lock().unwrap().push(BotEvent::Challenge(challenge));
        }

        async fn on_challenge_cancelled(&self, challenge: ChallengeEvent, _: &BotClient) {
            self.bot_events.lock().unwrap().push(BotEvent::ChallengeCanceled(challenge));
        }

        async fn on_challenge_declined(&self, challenge: ChallengeDeclinedEvent, _: &BotClient) {
            self.bot_events.lock().unwrap().push(BotEvent::ChallengeDeclined(challenge));
        }

        async fn on_game_full(&self, _: GameId, game_full: GameFullEvent, _: &BotClient) {
            self.game_events.lock().unwrap().push(GameEvent::GameFull(game_full))
        }

        async fn on_game_state(&self, _: GameId, state: GameStateEvent, _: &BotClient) {
            self.game_events.lock().unwrap().push(GameEvent::GameState(state))
        }

        async fn on_chat_line(&self, _: GameId, chat_line: ChatLineEvent, _: &BotClient) {
            self.game_events.lock().unwrap().push(GameEvent::ChatLine(chat_line))
        }

        async fn on_opponent_gone(&self, _: GameId, opponent_gone: OpponentGoneEvent,
                _: &BotClient) {
            self.game_events.lock().unwrap().push(GameEvent::OpponentGone(opponent_gone))
        }
    }

    fn create_mock_bot() -> (MockBot, Arc<Mutex<Vec<BotEvent>>>, Arc<Mutex<Vec<GameEvent>>>) {
        let bot_events = Arc::new(Mutex::new(Vec::new()));
        let game_events = Arc::new(Mutex::new(Vec::new()));
        let mock_bot = MockBot {
            bot_events: Arc::clone(&bot_events),
            game_events: Arc::clone(&game_events)
        };

        (mock_bot, bot_events, game_events)
    }

    fn test_game_event_info(id: &str) -> GameStartFinishEvent {
        GameStartFinishEvent {
            id: Some(id.to_owned()),
            source: None,
            status: None,
            winner: None,
            compat: None
        }
    }

    fn test_challenge(id: &str) -> ChallengeEvent {
        ChallengeEvent {
            id: id.to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: User {
                rating: None,
                provisional: None,
                online: None,
                id: "testUserId".to_owned(),
                name: "testUserName".to_owned(),
                title: None,
                patron: None,
            },
            dest_user: None,
            variant: None,
            rated: false,
            speed: Speed::UltraBullet,
            time_control: TimeControl::Unlimited,
            color: ChallengeColor::White,
            perf: ChallengePerf {
                icon: None,
                name: None
            },
            direction: None,
            initial_fen: None,
            decline_reason: None,
            decline_reason_key: None,
        }
    }

    #[rstest]
    #[case::empty(vec![])]
    #[case::on_game_start(vec![
        BotEvent::GameStart(test_game_event_info("testGameStartId"))
    ])]
    #[case::on_game_finish(vec![
        BotEvent::GameFinish(test_game_event_info("testGameFinishId"))
    ])]
    #[case::challenge(vec![
        BotEvent::Challenge(test_challenge("testChallengeId"))
    ])]
    #[case::challenge_canceled(vec![
        BotEvent::ChallengeCanceled(test_challenge("testChallengeCanceledId"))
    ])]
    #[case::challenge_declined(vec![
        BotEvent::ChallengeDeclined(ChallengeDeclinedEvent {
            id: "testChallengeDeclined".to_owned()
        })
    ])]
    #[case::multiple_events(vec![
        BotEvent::GameStart(test_game_event_info("firstEventId")),
        BotEvent::Challenge(test_challenge("secondEventId")),
        BotEvent::GameStart(test_game_event_info("thirdEventId"))
    ])]
    fn correct_events_are_called_on_bot(#[case] events: Vec<BotEvent>) {
        let (bot, tracked_events, _) = create_mock_bot();
        let event_results = events.iter()
            .cloned()
            .map(Ok)
            .collect::<Vec<Result<_, &str>>>();
        let stream = stream::iter(event_results);
        let mock_client = BotClientBuilder::new().with_token("").build().unwrap();

        tokio_test::block_on(run_with_event_stream(bot, stream, &mock_client));

        let tracked_events = tracked_events.lock().unwrap();

        assert_that!(tracked_events.deref()).contains_exactly_in_given_order(events);
    }

    #[test]
    fn game_start_event_with_game_id_causes_query_of_game_event_stream() {
        tokio_test::block_on(async {
            let (client, server) = test_util::setup_wiremock_test().await;
            let (bot, _, tracked_events) = create_mock_bot();

            Mock::given(method("GET"))
                .and(path("/bot/game/stream/testId"))
                .respond_with(ResponseTemplate::new(200)
                    .set_body_string("{\
                        \"type\": \"chatLine\",\
                        \"room\": \"player\",\
                        \"username\": \"testUserName\",\
                        \"text\": \"testText\"\
                    }\n"))
                .expect(1)
                .mount(&server)
                .await;
            let stream = stream::once(async {
                Ok::<_, &str>(BotEvent::GameStart(GameStartFinishEvent {
                    id: Some("testId".to_owned()),
                    source: None,
                    status: None,
                    winner: None,
                    compat: None,
                }))
            });

            run_with_event_stream(bot, stream, &client).await;

            let tracked_events = tracked_events.lock().unwrap();
            let expected_event = ChatLineEvent {
                room: ChatRoom::Player,
                username: "testUserName".to_string(),
                text: "testText".to_string()
            };

            assert_that!(tracked_events.deref())
                .contains_exactly_in_given_order([GameEvent::ChatLine(expected_event)]);
        });
    }
}
