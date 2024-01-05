use serde::Deserialize;
use serde_json::Value;
use std::hash::{Hash, Hasher};

use crate::model::game::Clock;

pub mod user;
pub mod game;
pub mod challenge;
pub mod bot_event;
pub(crate) mod request;

/// A Chess move in UCI notation.
pub type Move = String;

/// A space-separated list of Chess moves in UCI notation.
pub type Moves = String;
pub type Url = String;
pub type Milliseconds = i64;
pub type Seconds = i32;
pub type Days = i32;
pub type Timestamp = i64;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct Compat {
    // TODO Option<bool> correct?
    pub bot: Option<bool>,
    pub board: Option<bool>
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TimeControl {
    Clock(Clock),
    #[serde(rename_all = "camelCase")]
    Correspondence {
        // TODO really optional?
        days_per_turn: Option<Days>
    },
    Unlimited
}

struct AnyRef<'reference>(&'reference Value);

impl<'reference> Hash for AnyRef<'reference> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.0 {
            Value::Null => 0u8.hash(state),
            Value::Bool(value) => value.hash(state),
            Value::Number(value) => value.hash(state),
            Value::String(value) => value.hash(state),
            Value::Array(value) => value.iter()
                .map(AnyRef)
                .for_each(|item| item.hash(state)),
            Value::Object(value) => value.iter()
                .map(|(key, value)| (key, AnyRef(value)))
                .for_each(|item| item.hash(state))
        }
    }
}

/// Represents any JSON-[Value]. The wrapper allows for a [Hash] implementation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Any(pub Value);

impl Hash for Any {
    fn hash<H: Hasher>(&self, state: &mut H) {
        AnyRef(&self.0).hash(state)
    }
}
