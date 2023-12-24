use crate::model::challenge::DeclineReason;
use crate::model::game::chat::ChatRoom;

use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct DeclineRequest {

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reason: Option<DeclineReason>
}

#[derive(Serialize)]
pub(crate) struct SendChatMessageRequest {
    pub(crate) room: ChatRoom,
    pub(crate) text: String
}

#[cfg(test)]
mod tests {

    use kernal::prelude::*;

    use rstest::rstest;

    use crate::model::challenge::DeclineReason;
    use crate::model::request::DeclineRequest;

    #[test]
    fn serialize_decline_request_without_reason() {
        let decline_request = DeclineRequest {
            reason: None
        };

        let serialized = serde_json::to_string(&decline_request).unwrap();

        assert_that!(serialized).is_equal_to("{}".to_owned());
    }

    #[rstest]
    #[case(DeclineReason::Generic, r#"{"reason":"generic"}"#)]
    #[case(DeclineReason::Later, r#"{"reason":"later"}"#)]
    #[case(DeclineReason::TooFast, r#"{"reason":"tooFast"}"#)]
    #[case(DeclineReason::TooSlow, r#"{"reason":"tooSlow"}"#)]
    #[case(DeclineReason::TimeControl, r#"{"reason":"timeControl"}"#)]
    #[case(DeclineReason::Rated, r#"{"reason":"rated"}"#)]
    #[case(DeclineReason::Casual, r#"{"reason":"casual"}"#)]
    #[case(DeclineReason::Standard, r#"{"reason":"standard"}"#)]
    #[case(DeclineReason::Variant, r#"{"reason":"variant"}"#)]
    #[case(DeclineReason::NoBot, r#"{"reason":"noBot"}"#)]
    #[case(DeclineReason::OnlyBot, r#"{"reason":"onlyBot"}"#)]
    fn serialize_decline_request_with_reason(
        #[case] reason: DeclineReason, #[case] expected_json: &str) {
        let decline_request = DeclineRequest {
            reason: Some(reason)
        };

        let serialized = serde_json::to_string(&decline_request).unwrap();

        assert_that!(serialized).is_equal_to(expected_json.to_owned());
    }
}
