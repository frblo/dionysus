use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("room not found")]
    NotFound,

    #[error("already exists")]
    AlreadyExists,

    #[error(transparent)]
    StorageError(#[from] StorageError),

    #[error(transparent)]
    DecodingError(#[from] yrs::encoding::read::Error),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage internal error")]
    Internal {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
