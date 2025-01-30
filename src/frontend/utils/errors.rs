use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display};

pub type FrontendResult<T> = Result<T, FrontendError>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FrontendError(String);

impl FrontendError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn message(self) -> String {
        self.0
    }

    pub fn popup(self) {
        // TODO: show the error as popup and log it
        todo!();
    }
}

impl Display for FrontendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for FrontendError {}

#[cfg(feature = "ssr")]
impl From<reqwest::Error> for FrontendError {
    fn from(value: reqwest::Error) -> Self {
        Self(value.to_string())
    }
}
#[cfg(not(feature = "ssr"))]
impl From<gloo_net::Error> for FrontendError {
    fn from(value: gloo_net::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<url::ParseError> for FrontendError {
    fn from(value: url::ParseError) -> Self {
        Self(value.to_string())
    }
}
