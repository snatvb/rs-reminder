use prisma_client_rust::QueryError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Database query error")]
    DatabaseQueryError(#[from] QueryError),
    #[error("Request bot error")]
    RequestError(#[from] teloxide::RequestError),
}

pub type StateResult<T> = std::result::Result<T, StateError>;
