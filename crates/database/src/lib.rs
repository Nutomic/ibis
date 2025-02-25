use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use url::Url;

pub mod common;
#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod error;
#[cfg(feature = "ssr")]
pub mod impls;
#[cfg(feature = "ssr")]
pub mod scheduled_tasks;
#[cfg(feature = "ssr")]
mod schema;
#[cfg(feature = "ssr")]
pub mod utils;

#[repr(transparent)]
#[cfg_attr(
    feature = "ssr",
    derive(diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)
)]
#[cfg_attr(feature = "ssr", diesel(sql_type = diesel::sql_types::Text))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct DbUrl(pub Box<Url>);

impl Display for DbUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.clone().0.fmt(f)
    }
}

impl DbUrl {
    pub fn inner(&self) -> &Url {
        &self.0
    }
}
