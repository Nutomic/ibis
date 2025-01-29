use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display};

pub type FrontendResult<T> = Result<T, FrontendError>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrontendError(pub String);

impl Display for FrontendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for FrontendError {}

#[cfg(feature = "ssr")]
impl From<reqwest::Error> for FrontendError {
    fn from(value: reqwest::Error) -> Self {
        todo!()
    }
}
#[cfg(not(feature = "ssr"))]
impl From<gloo_net::Error> for FrontendError {
    fn from(value: gloo_net::Error) -> Self {
        todo!()
    }
}

/*
impl Render for FrontendError {
    type State = StringState;

    fn build(self) -> Self::State {
        todo!()
    }

    fn rebuild(self, state: &mut Self::State) {
        todo!()
    }
}

impl RenderHtml for FrontendError {
    type AsyncOutput = String;

    const MIN_LENGTH: usize = 0;

    fn dry_resolve(&mut self) {
        todo!()
    }

    async fn resolve(self) -> Self::AsyncOutput {
        todo!()
    }

    fn to_html_with_buf(
        self,
        buf: &mut String,
        position: &mut leptos::tachys::view::Position,
        escape: bool,
        mark_branches: bool,
    ) {
        todo!()
    }

    fn hydrate<const FROM_SERVER: bool>(
        self,
        cursor: &leptos::tachys::hydration::Cursor,
        position: &leptos::tachys::view::PositionState,
    ) -> Self::State {
        todo!()
    }
}

*/
