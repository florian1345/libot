use std::fmt::Debug;
use std::pin::pin;
use std::sync::Arc;

use futures::Stream;
use futures::stream::StreamExt;

use ndjson_stream::config::{EmptyLineHandling, NdjsonConfig};

use reqwest::Method;

use tokio::task;
use model::challenge::{Challenge, ChallengeDeclined};

use crate::client::BotClient;
use crate::context::{BotContext, GameContext};
use crate::error::LibotResult;
use crate::model::bot_event::{BotEvent, GameStartFinish};
use crate::model::game::{Color, GameId, GameInfo};
use crate::model::game::event::{ChatLineEvent, GameEvent, GameStateEvent, OpponentGoneEvent};
use crate::model::user::UserId;

pub mod model;
pub mod error;
pub mod client;
pub mod context;

#[cfg(test)]
pub(crate) mod test_util;

#[async_trait::async_trait]
pub trait Bot : Sync {

    async fn on_game_start(&self, _context: &BotContext, _game: GameStartFinish,
        _client: &BotClient) { }

    async fn on_game_finish(&self, _context: &BotContext, _game: GameStartFinish,
        _client: &BotClient) { }

    async fn on_challenge(&self, _context: &BotContext, _challenge: Challenge,
        _client: &BotClient) { }

    async fn on_challenge_cancelled(&self, _context: &BotContext, _challenge: Challenge,
        _client: &BotClient) { }

    async fn on_challenge_declined(&self, _context: &BotContext, _challenge: ChallengeDeclined,
        _client: &BotClient) { }

    async fn on_game_state(&self, _context: &GameContext, _state: GameStateEvent,
        _client: &BotClient) { }

    async fn on_chat_line(&self, _context: &GameContext, _chat_line: ChatLineEvent,
        _client: &BotClient) { }

    async fn on_opponent_gone(&self, _context: &GameContext, _opponent_gone: OpponentGoneEvent,
        _client: &BotClient) { }
}

const EVENT_PATH: &str = "/stream/event";

fn game_event_path(game_id: &GameId) -> String {
    format!("/bot/game/stream/{}", game_id)
}

fn color_of(user_id: &UserId, game_info: &GameInfo) -> Option<Color> {
    let is_white = game_info.white.id.iter().any(|white| white == user_id);
    let is_black = game_info.black.id.iter().any(|black| black == user_id);

    if is_white {
        Some(Color::White)
    }
    else if is_black {
        Some(Color::Black)
    }
    else {
        None
    }
}

async fn process_game_event(event: GameEvent, game_context: &GameContext, bot: &impl Bot,
        client: &BotClient) {
    // TODO enable error handling
    match event {
        GameEvent::GameFull(_) => panic!(), // TODO proper error handling
        GameEvent::GameState(state) =>
            bot.on_game_state(game_context, state, client).await,
        GameEvent::ChatLine(chat_line) =>
            bot.on_chat_line(game_context, chat_line, client).await,
        GameEvent::OpponentGone(opponent_gone) =>
            bot.on_opponent_gone(game_context, opponent_gone, client).await,
    }
}

async fn run_with_game_event_stream<E>(bot: Arc<impl Bot + Send + 'static>,
    mut event_stream: impl Stream<Item = Result<GameEvent, E>>, client: BotClient, bot_id: UserId)
where
    E: Debug + Send + 'static
{
    let game_context;
    let mut event_stream = pin!(event_stream);

    match event_stream.next().await {
        Some(Ok(GameEvent::GameFull(game_full))) => {
            let bot_color = color_of(&bot_id, &game_full.info);

            game_context = GameContext {
                bot_color,
                bot_id: bot_id.clone(),
                info: game_full.info
            };

            bot.on_game_state(&game_context, game_full.state, &client).await
        },
        Some(_) => panic!(), // TODO proper error handling
        None => return
    };

    let game_context = Arc::new(game_context);

    event_stream.map(|record| {
        let bot = Arc::clone(&bot);
        let client = client.clone();
        let game_context = Arc::clone(&game_context);

        task::spawn(async move {
            process_game_event(
                record.unwrap(), game_context.as_ref(), bot.as_ref(), &client).await;
        })
    }).for_each_concurrent(None, |join_handle| async { join_handle.await.unwrap() }).await;
}

async fn process_bot_event(event: BotEvent, bot: Arc<impl Bot + Send + 'static>,
        client: BotClient, context: &BotContext) {
    // TODO enable error handling
    match event {
        BotEvent::GameStart(game) => {
            let game_id = game.id.clone();
            bot.as_ref().on_game_start(context, game, &client).await;

            if let Some(game_id) = game_id {
                let event_path = game_event_path(&game_id);

                // TODO enable error handling
                if let Ok(response) = client.send_request(Method::GET, &event_path).await {
                    let stream =
                        ndjson_stream::from_fallible_stream_with_config::<GameEvent, _>(
                            response.bytes_stream(), ndjson_config());

                    run_with_game_event_stream(bot, stream, client, context.bot_id.clone()).await
                }
            }
        },
        BotEvent::GameFinish(game) =>
            bot.as_ref().on_game_finish(context, game, &client).await,
        BotEvent::Challenge(challenge) =>
            bot.as_ref().on_challenge(context, challenge, &client).await,
        BotEvent::ChallengeCanceled(challenge) =>
            bot.as_ref().on_challenge_cancelled(context, challenge, &client).await,
        BotEvent::ChallengeDeclined(challenge) =>
            bot.as_ref().on_challenge_declined(context, challenge, &client).await
    }
}

async fn run_with_event_stream<E>(bot: Arc<impl Bot + Send + 'static>,
    event_stream: impl Stream<Item = Result<BotEvent, E>>, client: BotClient, bot_id: UserId)
where
    E: Debug + Send + 'static
{
    let context = Arc::new(BotContext {
        bot_id
    });

    event_stream.map(move |record| {
        let bot = Arc::clone(&bot);
        let client = client.clone();
        let context = Arc::clone(&context);

        task::spawn(async move {
            process_bot_event(record.unwrap(), bot, client, context.as_ref()).await;
        })
    }).for_each_concurrent(None, |join_handle| async { join_handle.await.unwrap() }).await;
}

pub async fn run(bot: impl Bot + Send + 'static, client: BotClient) -> LibotResult<()> {
    let bot_id = client.get_my_profile().await.unwrap().id;
    let response = client.send_request(Method::GET, EVENT_PATH).await?;
    let stream =
        ndjson_stream::from_fallible_stream_with_config::<BotEvent, _>(
            response.bytes_stream(), ndjson_config());
    let bot = Arc::new(bot);

    #[allow(clippy::unit_arg)]
    Ok(run_with_event_stream(bot, stream, client, bot_id).await)
}

fn ndjson_config() -> NdjsonConfig {
    NdjsonConfig::default()
        .with_empty_line_handling(EmptyLineHandling::IgnoreEmpty)
}

#[cfg(test)]
mod tests {

    use std::iter;
    use std::ops::Deref;
    use std::sync::{Arc, Mutex};

    use futures::stream;

    use kernal::prelude::*;

    use rstest::rstest;

    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    use crate::client::BotClientBuilder;
    use crate::model::TimeControl;
    use crate::model::challenge::{ChallengeColor, ChallengePerf, ChallengeStatus};
    use crate::model::game::{GamePerf, GameStatus, Speed, Variant};
    use crate::model::game::chat::{ChatLine, ChatRoom};
    use crate::model::game::event::{GameEventPlayer, GameFullEvent};
    use crate::model::user::User;

    use super::*;

    struct MockBot {
        bot_events: Arc<Mutex<Vec<BotEvent>>>,
        game_events: Arc<Mutex<Vec<(GameContext, GameEvent)>>>
    }

    #[async_trait::async_trait]
    impl Bot for MockBot {
        async fn on_game_start(&self, _: &BotContext, game: GameStartFinish, _: &BotClient) {
            self.bot_events.lock().unwrap().push(BotEvent::GameStart(game));
        }

        async fn on_game_finish(&self, _: &BotContext, game: GameStartFinish, _: &BotClient) {
            self.bot_events.lock().unwrap().push(BotEvent::GameFinish(game));
        }

        async fn on_challenge(&self, _: &BotContext, challenge: Challenge, _: &BotClient) {
            self.bot_events.lock().unwrap().push(BotEvent::Challenge(challenge));
        }

        async fn on_challenge_cancelled(&self, _: &BotContext, challenge: Challenge,
                _: &BotClient) {
            self.bot_events.lock().unwrap().push(BotEvent::ChallengeCanceled(challenge));
        }

        async fn on_challenge_declined(&self, _: &BotContext, challenge: ChallengeDeclined,
                _: &BotClient) {
            self.bot_events.lock().unwrap().push(BotEvent::ChallengeDeclined(challenge));
        }

        async fn on_game_state(&self, context: &GameContext, state: GameStateEvent, _: &BotClient) {
            self.game_events.lock().unwrap().push((context.clone(), GameEvent::GameState(state)))
        }

        async fn on_chat_line(&self, context: &GameContext, chat_line: ChatLineEvent,
                _: &BotClient) {
            self.game_events.lock().unwrap().push((context.clone(), GameEvent::ChatLine(chat_line)))
        }

        async fn on_opponent_gone(&self, context: &GameContext, opponent_gone: OpponentGoneEvent,
                _: &BotClient) {
            self.game_events.lock().unwrap()
                .push((context.clone(), GameEvent::OpponentGone(opponent_gone)))
        }
    }

    fn create_mock_bot() -> (MockBot, Arc<Mutex<Vec<BotEvent>>>,
            Arc<Mutex<Vec<(GameContext, GameEvent)>>>) {
        let bot_events = Arc::new(Mutex::new(Vec::new()));
        let game_events = Arc::new(Mutex::new(Vec::new()));
        let mock_bot = MockBot {
            bot_events: Arc::clone(&bot_events),
            game_events: Arc::clone(&game_events)
        };

        (mock_bot, bot_events, game_events)
    }

    fn test_game_event_info(id: &str) -> GameStartFinish {
        GameStartFinish {
            id: Some(id.to_owned()),
            source: None,
            status: None,
            winner: None,
            compat: None
        }
    }

    fn test_challenge(id: &str) -> Challenge {
        Challenge {
            id: id.to_owned(),
            url: "testUrl".to_owned(),
            status: ChallengeStatus::Created,
            challenger: User {
                rating: None,
                provisional: false,
                online: false,
                id: "testUserId".to_owned(),
                name: "testUserName".to_owned(),
                title: None,
                patron: false,
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
        BotEvent::ChallengeDeclined(ChallengeDeclined {
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

        tokio_test::block_on(run_with_event_stream(
            Arc::new(bot), stream, mock_client, "testId".to_owned()));

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
                        \"type\": \"gameFull\",\
                        \"id\": \"testId\",\
                        \"variant\": { },\
                        \"clock\": null,\
                        \"speed\": \"blitz\",\
                        \"perf\": { },\
                        \"rated\": false,\
                        \"createdAt\": 1234,\
                        \"white\": { },\
                        \"black\": { },\
                        \"initialFen\": \"testInitialFen\",\
                        \"state\": {\
                            \"type\": \"gameState\",\
                            \"moves\": \"\",\
                            \"wtime\": 120000,\
                            \"btime\": 120000,\
                            \"winc\": 0,\
                            \"binc\": 0,\
                            \"status\": \"created\"\
                        }\
                    }\n"))
                .expect(1)
                .mount(&server)
                .await;
            let stream = stream::once(async {
                Ok::<_, &str>(BotEvent::GameStart(GameStartFinish {
                    id: Some("testId".to_owned()),
                    source: None,
                    status: None,
                    winner: None,
                    compat: None,
                }))
            });

            run_with_event_stream(Arc::new(bot), stream, client, "testId".to_owned()).await;

            let tracked_events = tracked_events.lock().unwrap();
            let expected_event = GameStateEvent {
                moves: "".to_string(),
                white_time: 120000,
                black_time: 120000,
                white_increment: 0,
                black_increment: 0,
                status: GameStatus::Created,
                winner: None,
                white_draw_offer: false,
                black_draw_offer: false,
                white_take_back_proposal: false,
                black_take_back_proposal: false,
            };

            assert_that!(tracked_events.deref()).has_length(1);
            assert_that!(&tracked_events.deref()[0].1)
                .is_equal_to(&GameEvent::GameState(expected_event.clone()));
        });
    }

    fn player_with_id(id: &str) -> GameEventPlayer {
        GameEventPlayer {
            ai_level: None,
            id: Some(id.to_owned()),
            name: None,
            title: None,
            rating: None,
            provisional: None
        }
    }

    fn game_state_event(moves: &str) -> GameStateEvent {
        GameStateEvent {
            moves: moves.to_string(),
            white_time: 1,
            black_time: 2,
            white_increment: 3,
            black_increment: 4,
            status: GameStatus::Created,
            winner: None,
            white_draw_offer: false,
            black_draw_offer: false,
            white_take_back_proposal: false,
            black_take_back_proposal: false,
        }
    }

    #[rstest]
    #[case::no_further_events(vec![])]
    #[case::game_state_event(vec![
        GameEvent::GameState(game_state_event("testMoves2"))
    ])]
    #[case::chat_line(vec![
        GameEvent::ChatLine(ChatLineEvent {
            room: ChatRoom::Player,
            chat_line: ChatLine {
                username: "testUsername".to_owned(),
                text: "testText".to_owned()
            }
        })
    ])]
    #[case::opponent_gone(vec![
        GameEvent::OpponentGone(OpponentGoneEvent {
            gone: true,
            claim_win_in_seconds: Some(30)
        })
    ])]
    fn correct_game_events_are_called_on_bot(#[case] events: Vec<GameEvent>) {
        let game_info = GameInfo {
            id: "testGameId".to_string(),
            variant: Some(Variant::Standard),
            clock: None,
            speed: Speed::Bullet,
            perf: GamePerf {
                name: None,
            },
            rated: false,
            created_at: 0,
            white: player_with_id("testWhiteId"),
            black: player_with_id("testBlackId"),
            initial_fen: "testInitialFen".to_string(),
            tournament_id: None,
        };
        let first_state_event = game_state_event("testMoves1");

        let (bot, _, tracked_events) = create_mock_bot();
        let event_results = iter::once(
                GameEvent::GameFull(GameFullEvent {
                    info: game_info.clone(),
                    state: first_state_event.clone(),
                }))
            .chain(events.iter().cloned())
            .map(Ok)
            .collect::<Vec<Result<_, &str>>>();
        let stream = stream::iter(event_results);
        let mock_client = BotClientBuilder::new().with_token("").build().unwrap();
        let bot_id = "testId".to_owned();

        tokio_test::block_on(run_with_game_event_stream(
            Arc::new(bot), stream, mock_client, bot_id.clone()));

        let tracked_events = tracked_events.lock().unwrap();
        let expected_context = GameContext {
            bot_color: None,
            bot_id,
            info: game_info
        };
        let expected_events = events.into_iter()
            .map(|event| (expected_context.clone(), event))
            .collect::<Vec<_>>();

        assert_that!(tracked_events.deref())
            .has_length(expected_events.len() + 1)
            .starts_with([(expected_context, GameEvent::GameState(first_state_event))])
            .ends_with(expected_events);
    }

    #[rstest]
    #[case::neither("testWhiteId", "testBlackId", "testBotId", None)]
    #[case::white("testBotId", "testBlackId", "testBotId", Some(Color::White))]
    #[case::black("testWhiteId", "testBotId", "testBotId", Some(Color::Black))]
    fn game_context_has_correct_bot_color(
            #[case] white_id: &str,
            #[case] black_id: &str,
            #[case] bot_id: &str,
            #[case] expected_bot_color: Option<Color>) {
        let game_info = GameInfo {
            id: "testGameId".to_string(),
            variant: Some(Variant::Standard),
            clock: None,
            speed: Speed::Classical,
            perf: GamePerf {
                name: None,
            },
            rated: false,
            created_at: 0,
            white: player_with_id(white_id),
            black: player_with_id(black_id),
            initial_fen: "testInitialFen".to_string(),
            tournament_id: None,
        };
        let state_event = game_state_event("testMoves");

        let (bot, _, tracked_events) = create_mock_bot();
        let stream = stream::once(async {
            Ok::<_, &str>(GameEvent::GameFull(
                GameFullEvent {
                    info: game_info.clone(),
                    state: state_event
                }))
        });
        let mock_client = BotClientBuilder::new().with_token("").build().unwrap();

        tokio_test::block_on(run_with_game_event_stream(
            Arc::new(bot), stream, mock_client, bot_id.to_owned()));

        let tracked_events = tracked_events.lock().unwrap();

        assert_that!(tracked_events.deref()[0].0.bot_color).is_equal_to(expected_bot_color);
    }
}
