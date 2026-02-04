use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("room not found")]
    NotFound,

    #[error("already exists")]
    AlreadyExists,

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error(transparent)]
    Decoding(#[from] yrs::encoding::read::Error),

    #[error("storage internal error")]
    Backend {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
