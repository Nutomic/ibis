use leptos::prelude::*;
use log::warn;
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
}

pub trait FrontendResultExt<T> {
    fn error_popup<F>(self, on_success: F)
    where
        F: FnOnce(T);
}

impl<T> FrontendResultExt<T> for FrontendResult<T> {
    fn error_popup<F>(self, on_success: F)
    where
        F: FnOnce(T),
    {
        match self {
            Ok(o) => on_success(o),
            Err(e) => {
                warn!("{e}");
                if let Some(error_popup) = use_context::<WriteSignal<Option<String>>>() {
                    error_popup.set(Some(e.0));
                }
            }
        }
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

impl From<serde_urlencoded::ser::Error> for FrontendError {
    fn from(value: serde_urlencoded::ser::Error) -> Self {
        Self(value.to_string())
    }
}
