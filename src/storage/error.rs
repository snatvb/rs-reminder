use prisma_client_rust::QueryError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database query error")]
    DatabaseQueryError(#[from] QueryError),
    #[error("Word already exists")]
    WordAlreadyExists,
}

pub type StorageResult<T> = std::result::Result<T, StorageError>;
