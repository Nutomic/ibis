use activitypub_federation::http_signatures::{generate_actor_keypair, Keypair};
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::{Output, ToSql},
    sql_types::Text,
};
use error::BackendResult;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    sync::LazyLock,
};
use url::Url;

mod common;
mod config;
mod error;
mod impls;
mod schema;

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DbUrl(pub(crate) Box<Url>);

impl ToSql<Text, Pg> for DbUrl {
    fn to_sql(&self, out: &mut Output<Pg>) -> diesel::serialize::Result {
        <std::string::String as ToSql<Text, Pg>>::to_sql(&self.0.to_string(), &mut out.reborrow())
    }
}

impl<DB: Backend> FromSql<Text, DB> for DbUrl
where
    String: FromSql<Text, DB>,
{
    fn from_sql(value: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let str = String::from_sql(value)?;
        Ok(DbUrl(Box::new(Url::parse(&str)?)))
    }
}

impl Display for DbUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.clone().0.fmt(f)
    }
}

#[expect(clippy::from_over_into)]
impl Into<DbUrl> for Url {
    fn into(self) -> DbUrl {
        DbUrl(Box::new(self))
    }
}
#[expect(clippy::from_over_into)]
impl Into<Url> for DbUrl {
    fn into(self) -> Url {
        *self.0
    }
}

/// Use a single static keypair during testing which is signficantly faster than
/// generating dozens of keys from scratch.
pub fn generate_keypair() -> BackendResult<Keypair> {
    if cfg!(debug_assertions) {
        static KEYPAIR: LazyLock<Keypair> =
            LazyLock::new(|| generate_actor_keypair().expect("generate keypair"));
        Ok(KEYPAIR.clone())
    } else {
        Ok(generate_actor_keypair()?)
    }
}
