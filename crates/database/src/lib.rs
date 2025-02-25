use activitypub_federation::{
    fetch::{collection_id::CollectionId, object_id::ObjectId},
    http_signatures::{generate_actor_keypair, Keypair},
    traits::{Collection, Object},
};
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

pub mod common;
pub mod config;
pub mod error;
pub mod impls;
pub mod scheduled_tasks;
mod schema;

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DbUrl(pub Box<Url>);

impl DbUrl {
    pub fn inner(&self) -> &Url {
        &self.0
    }
}

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

#[cfg(feature = "ssr")]
impl<T> From<DbUrl> for ObjectId<T>
where
    T: Object + Send + 'static,
    for<'de2> <T as Object>::Kind: Deserialize<'de2>,
{
    fn from(value: DbUrl) -> Self {
        let url: Url = value.into();
        ObjectId::from(url)
    }
}

#[cfg(feature = "ssr")]
impl<Kind> From<ObjectId<Kind>> for DbUrl
where
    Kind: Object + Send + 'static,
    for<'de2> <Kind as Object>::Kind: serde::Deserialize<'de2>,
{
    fn from(id: ObjectId<Kind>) -> Self {
        DbUrl(Box::new(id.into()))
    }
}
#[cfg(feature = "ssr")]
impl<T> From<DbUrl> for CollectionId<T>
where
    T: Collection + Send + 'static,
    for<'de2> <T as Collection>::Kind: Deserialize<'de2>,
{
    fn from(value: DbUrl) -> Self {
        let url: Url = value.into();
        CollectionId::from(url)
    }
}

#[cfg(feature = "ssr")]
impl<T> From<CollectionId<T>> for DbUrl
where
    T: Collection,
    for<'de2> <T as Collection>::Kind: Deserialize<'de2>,
{
    fn from(value: CollectionId<T>) -> Self {
        let url: Url = value.into();
        url.into()
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
