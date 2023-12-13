use reqwest::{Error as ReqwestError, StatusCode};
use reqwest::header::InvalidHeaderValue;

use serde_json::Error as JsonError;

use thiserror::Error;

use crate::client::BotClient;

#[derive(Debug, Error)]
pub enum LibotRequestError {

    #[error("networking error: {0}")]
    ReqwestError(#[from] ReqwestError),

    #[error("error serializing body or deserializing response: {0}")]
    JsonError(#[from] JsonError),

    #[error("status {status} from API with body: {body:?}")]
    ApiError {
        status: StatusCode,
        body: Option<String>
    }
}

pub type LibotResult<T> = Result<T, LibotRequestError>;

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
