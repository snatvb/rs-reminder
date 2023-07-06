use teloxide::types::User;
use thiserror::Error;

use crate::storage::error::StorageError;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Storage error")]
    StorageError(#[from] StorageError),
    #[error("Request bot error")]
    RequestError(#[from] teloxide::RequestError),
    #[error("Word already exists")]
    WordAlreadyExists,
    #[error("Unexpected command")]
    UnexpectedCommand(String),
    #[error("Unexpected query")]
    UnexpectedQueryData(Option<String>, User),
    #[error("Incorrect word level: `{0}`")]
    IncorrectWordLevel(i32),
}

pub type StateResult<T> = std::result::Result<T, StateError>;
