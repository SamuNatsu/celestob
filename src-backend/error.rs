use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Anyhow error result wrapper
pub type Result<T> = std::result::Result<T, Error>;

/// Anyhow error response wrapper
pub struct Error(anyhow::Error);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal Server Error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}
