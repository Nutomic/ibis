use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::{Output, ToSql},
    sql_types::Text,
};
use serde::{Deserialize, Serialize};
use url::Url;

mod common;
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
