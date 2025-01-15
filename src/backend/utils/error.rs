use std::fmt::{Display, Formatter};

pub type MyResult<T> = Result<T, Error>;

#[derive(Debug)]
pub struct Error(pub anyhow::Error);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<T> From<T> for Error
where
    T: Into<anyhow::Error>,
{
    fn from(t: T) -> Self {
        Error(t.into())
    }
}

#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}", self.0),
        )
            .into_response()
    }
}
