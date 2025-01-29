use std::fmt::{Display, Formatter};

pub type BackendResult<T> = Result<T, BackendError>;

#[derive(Debug)]
pub struct BackendError(pub anyhow::Error);

impl Display for BackendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<T> From<T> for BackendError
where
    T: Into<anyhow::Error>,
{
    fn from(t: T) -> Self {
        BackendError(t.into())
    }
}

#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for BackendError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}", self.0),
        )
            .into_response()
    }
}
