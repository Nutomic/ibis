use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::traits::{Collection, Object};
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use url::Url;

/// Copied from lemmy, could be moved into common library
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DbUrl(pub(crate) Box<Url>);

// TODO: Lemmy doesnt need this, but for some reason derive fails to generate it
impl FromSql<diesel::sql_types::Text, Pg> for DbUrl {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        todo!()
    }
}

impl Display for DbUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.clone().0.fmt(f)
    }
}

// the project doesnt compile with From
#[allow(clippy::from_over_into)]
impl Into<DbUrl> for Url {
    fn into(self) -> DbUrl {
        DbUrl(Box::new(self))
    }
}
#[allow(clippy::from_over_into)]
impl Into<Url> for DbUrl {
    fn into(self) -> Url {
        *self.0
    }
}

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

impl<T> From<ObjectId<T>> for DbUrl
where
    T: Object,
    for<'de2> <T as Object>::Kind: Deserialize<'de2>,
{
    fn from(value: ObjectId<T>) -> Self {
        let url: Url = value.into();
        url.into()
    }
}

impl Deref for DbUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
