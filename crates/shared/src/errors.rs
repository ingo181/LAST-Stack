use thiserror::Error;

/// Unified error type across all services.
/// Axum handlers map this to HTTP status codes.
/// Leptos components match on the variant for UI error messages.
#[derive(Debug, Error, Clone)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("kafka error: {0}")]
    Kafka(String),

    #[error("invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("serialisation error: {0}")]
    Serialisation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// HTTP status code — used by Axum IntoResponse impl in each service.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFound(_)           => 404,
            Self::Validation(_)         => 422,
            Self::Unauthorized          => 401,
            Self::Forbidden(_)          => 403,
            Self::Conflict(_)           => 409,
            Self::InvalidTransition {..}=> 422,
            Self::Database(_)
            | Self::Kafka(_)
            | Self::Serialisation(_)
            | Self::Internal(_)         => 500,
        }
    }

    pub fn is_client_error(&self) -> bool {
        self.status_code() < 500
    }
}

// Convenience From impls so services can use `?` directly.
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialisation(e.to_string())
    }
}