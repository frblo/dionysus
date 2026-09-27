use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The actor is not permitted to perform this action.
    #[error("actor is not permitted to perform this action")]
    Denied,

    /// The action isn't a valid action.
    ///
    /// E.g. trying to grant room ownership to a global
    /// [`Guest`](crate::authz::GlobalRole::Guest).
    #[error("invalid authorization change: {0}")]
    Invalid(String),

    /// The backend just can't do this at all.
    #[error("operation not supported by the configured authorization backend")]
    Unsupported,

    /// The backend itself failed, so we don't actually know the answer.
    /// Callers should treat this as a denial.
    #[error("authorization backend error")]
    Backend {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::warn!(error = ?self, "authz error");

        let status = match &self {
            Error::Denied => StatusCode::FORBIDDEN,
            Error::Invalid(_) => StatusCode::BAD_REQUEST,
            Error::Unsupported => StatusCode::NOT_IMPLEMENTED,
            Error::Backend { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
