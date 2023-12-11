use wiremock::MockServer;

use crate::client::{BotClient, BotClientBuilder};

pub async fn setup_wiremock_test() -> (BotClient, MockServer) {
    let server = MockServer::start().await;
    let client = BotClientBuilder::new()
        .with_token("mock_token")
        .with_base_url(server.uri())
        .build()
        .unwrap();

    (client, server)
}
