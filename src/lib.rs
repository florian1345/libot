use std::fmt::Debug;

use futures::Stream;
use futures::stream::StreamExt;

use ndjson_stream::config::{EmptyLineHandling, NdjsonConfig};

use reqwest::Method;

use crate::client::BotClient;
use crate::error::LibotResult;
use crate::model::{Challenge, ChallengeDeclined, Event, GameEventInfo};

pub mod model;
pub mod error;
pub mod client;

#[async_trait::async_trait]
pub trait Bot {

    async fn on_game_start(&self, game: GameEventInfo, client: &BotClient);

    async fn on_game_finish(&self, game: GameEventInfo, client: &BotClient);

    async fn on_challenge(&self, challenge: Challenge, client: &BotClient);

    async fn on_challenge_cancelled(&self, challenge: Challenge, client: &BotClient);

    async fn on_challenge_declined(&self, challenge: ChallengeDeclined, client: &BotClient);
}

const EVENT_PATH: &str = "/stream/event";

async fn run_with_event_stream<E>(bot: impl Bot, event_stream: impl Stream<Item = Result<Event, E>>,
    client: &BotClient)
where
    E: Debug
{
    event_stream.for_each(|record| async {
        // TODO enable error handling
        match record.unwrap() {
            Event::GameStart(game) => bot.on_game_start(game, client).await,
            Event::GameFinish(game) => bot.on_game_finish(game, client).await,
            Event::Challenge(challenge) => bot.on_challenge(challenge, client).await,
            Event::ChallengeCanceled(challenge) =>
                bot.on_challenge_cancelled(challenge, client).await,
            Event::ChallengeDeclined(challenge) =>
                bot.on_challenge_declined(challenge, client).await
        }
    }).await
}

pub async fn run(bot: impl Bot, client: BotClient) -> LibotResult<()> {
    let response = client.send_request(Method::GET, EVENT_PATH).await?;
    let ndjson_config = NdjsonConfig::default()
        .with_empty_line_handling(EmptyLineHandling::IgnoreEmpty);
    let stream =
        ndjson_stream::from_fallible_stream_with_config::<Event, _>(
            response.bytes_stream(), ndjson_config);

    #[allow(clippy::unit_arg)]
    Ok(run_with_event_stream(bot, stream, &client).await)
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use std::sync::{Arc, Mutex};

    use futures::stream;
    use kernal::prelude::*;
    use rstest::rstest;

    use crate::client::BotClientBuilder;
    use crate::model::{ChallengeColor, ChallengePerf, ChallengeStatus, Speed, TimeControl, User};

    use super::*;

    struct EventTrackingBot {
        events: Arc<Mutex<Vec<Event>>>
    }

    #[async_trait::async_trait]
    impl Bot for EventTrackingBot {
        async fn on_game_start(&self, game: GameEventInfo, _: &BotClient) {
            self.events.lock().unwrap().push(Event::GameStart(game));
        }

        async fn on_game_finish(&self, game: GameEventInfo, _: &BotClient) {
            self.events.lock().unwrap().push(Event::GameFinish(game));
        }

        async fn on_challenge(&self, challenge: Challenge, _: &BotClient) {
            self.events.lock().unwrap().push(Event::Challenge(challenge));
        }

        async fn on_challenge_cancelled(&self, challenge: Challenge, _: &BotClient) {
            self.events.lock().unwrap().push(Event::ChallengeCanceled(challenge));
        }

        async fn on_challenge_declined(&self, challenge: ChallengeDeclined, _: &BotClient) {
            self.events.lock().unwrap().push(Event::ChallengeDeclined(challenge));
        }
    }

    fn test_game_event_info(id: &str) -> GameEventInfo {
        GameEventInfo {
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
        Event::GameStart(test_game_event_info("testGameStartId"))
    ])]
    #[case::on_game_finish(vec![
        Event::GameFinish(test_game_event_info("testGameFinishId"))
    ])]
    #[case::challenge(vec![
        Event::Challenge(test_challenge("testChallengeId"))
    ])]
    #[case::challenge_canceled(vec![
        Event::ChallengeCanceled(test_challenge("testChallengeCanceledId"))
    ])]
    #[case::challenge_declined(vec![
        Event::ChallengeDeclined(ChallengeDeclined {
            id: "testChallengeDeclined".to_owned()
        })
    ])]
    #[case::multiple_events(vec![
        Event::GameStart(test_game_event_info("firstEventId")),
        Event::Challenge(test_challenge("secondEventId")),
        Event::GameStart(test_game_event_info("thirdEventId"))
    ])]
    fn correct_events_are_called_on_bot(#[case] events: Vec<Event>) {
        let tracked_events = Arc::new(Mutex::new(Vec::new()));
        let bot = EventTrackingBot {
            events: Arc::clone(&tracked_events)
        };
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
}
