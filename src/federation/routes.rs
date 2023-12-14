use crate::database::article::DbArticle;
use crate::database::instance::DbInstance;
use crate::database::user::DbPerson;
use crate::database::MyDataHandle;
use crate::error::{Error, MyResult};
use crate::federation::activities::accept::Accept;
use crate::federation::activities::create_article::CreateArticle;
use crate::federation::activities::follow::Follow;
use crate::federation::activities::reject::RejectEdit;
use crate::federation::activities::update_local_article::UpdateLocalArticle;
use crate::federation::activities::update_remote_article::UpdateRemoteArticle;
use crate::federation::objects::article::ApubArticle;
use crate::federation::objects::articles_collection::{ArticleCollection, DbArticleCollection};
use crate::federation::objects::edits_collection::{ApubEditCollection, DbEditCollection};
use crate::federation::objects::instance::ApubInstance;
use crate::federation::objects::user::ApubUser;
use activitypub_federation::axum::inbox::{receive_activity, ActivityData};
use activitypub_federation::axum::json::FederationJson;
use activitypub_federation::config::Data;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::Actor;
use activitypub_federation::traits::Object;
use activitypub_federation::traits::{ActivityHandler, Collection};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use axum_macros::debug_handler;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

pub fn federation_routes() -> Router {
    Router::new()
        .route("/", get(http_get_instance))
        .route("/user/:name", get(http_get_person))
        .route("/all_articles", get(http_get_all_articles))
        .route("/article/:title", get(http_get_article))
        .route("/article/:title/edits", get(http_get_article_edits))
        .route("/inbox", post(http_post_inbox))
}

#[debug_handler]
async fn http_get_instance(
    data: Data<MyDataHandle>,
) -> MyResult<FederationJson<WithContext<ApubInstance>>> {
    let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
    let json_instance = local_instance.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(json_instance)))
}

#[debug_handler]
async fn http_get_person(
    Path(name): Path<String>,
    data: Data<MyDataHandle>,
) -> MyResult<FederationJson<WithContext<ApubUser>>> {
    let person = DbPerson::read_local_from_name(&name, &data)?.person;
    let json_person = person.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(json_person)))
}

#[debug_handler]
async fn http_get_all_articles(
    data: Data<MyDataHandle>,
) -> MyResult<FederationJson<WithContext<ArticleCollection>>> {
    let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
    let collection = DbArticleCollection::read_local(&local_instance, &data).await?;
    Ok(FederationJson(WithContext::new_default(collection)))
}

#[debug_handler]
async fn http_get_article(
    Path(title): Path<String>,
    data: Data<MyDataHandle>,
) -> MyResult<FederationJson<WithContext<ApubArticle>>> {
    let article = DbArticle::read_local_title(&title, &data.db_connection)?;
    let json = article.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(json)))
}

#[debug_handler]
async fn http_get_article_edits(
    Path(title): Path<String>,
    data: Data<MyDataHandle>,
) -> MyResult<FederationJson<WithContext<ApubEditCollection>>> {
    let article = DbArticle::read_local_title(&title, &data.db_connection)?;
    let json = DbEditCollection::read_local(&article, &data).await?;
    Ok(FederationJson(WithContext::new_default(json)))
}

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum InboxActivities {
    Follow(Follow),
    Accept(Accept),
    CreateArticle(CreateArticle),
    UpdateLocalArticle(UpdateLocalArticle),
    UpdateRemoteArticle(UpdateRemoteArticle),
    RejectEdit(RejectEdit),
}

#[debug_handler]
pub async fn http_post_inbox(
    data: Data<MyDataHandle>,
    activity_data: ActivityData,
) -> impl IntoResponse {
    receive_activity::<WithContext<InboxActivities>, UserOrInstance, MyDataHandle>(
        activity_data,
        &data,
    )
    .await
}

#[derive(Clone, Debug)]
pub enum UserOrInstance {
    User(DbPerson),
    Instance(DbInstance),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PersonOrInstance {
    Person(ApubUser),
    Instance(ApubInstance),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PersonOrInstanceType {
    Person,
    Group,
}

#[async_trait::async_trait]
impl Object for UserOrInstance {
    type DataType = MyDataHandle;
    type Kind = PersonOrInstance;
    type Error = Error;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        Some(match self {
            UserOrInstance::User(p) => p.last_refreshed_at,
            UserOrInstance::Instance(p) => p.last_refreshed_at,
        })
    }

    #[tracing::instrument(skip_all)]
    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Error> {
        let person = DbPerson::read_from_id(object_id.clone(), data).await?;
        Ok(match person {
            Some(o) => Some(UserOrInstance::User(o)),
            None => DbInstance::read_from_id(object_id, data)
                .await?
                .map(UserOrInstance::Instance),
        })
    }

    #[tracing::instrument(skip_all)]
    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), Error> {
        match self {
            UserOrInstance::User(p) => p.delete(data).await,
            UserOrInstance::Instance(p) => p.delete(data).await,
        }
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Error> {
        unimplemented!()
    }

    #[tracing::instrument(skip_all)]
    async fn verify(
        apub: &Self::Kind,
        expected_domain: &Url,
        data: &Data<Self::DataType>,
    ) -> Result<(), Error> {
        match apub {
            PersonOrInstance::Person(a) => DbPerson::verify(a, expected_domain, data).await,
            PersonOrInstance::Instance(a) => DbInstance::verify(a, expected_domain, data).await,
        }
    }

    #[tracing::instrument(skip_all)]
    async fn from_json(apub: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Error> {
        Ok(match apub {
            PersonOrInstance::Person(p) => {
                UserOrInstance::User(DbPerson::from_json(p, data).await?)
            }
            PersonOrInstance::Instance(p) => {
                UserOrInstance::Instance(DbInstance::from_json(p, data).await?)
            }
        })
    }
}

impl Actor for UserOrInstance {
    fn id(&self) -> Url {
        match self {
            UserOrInstance::User(u) => u.id(),
            UserOrInstance::Instance(c) => c.id(),
        }
    }

    fn public_key_pem(&self) -> &str {
        match self {
            UserOrInstance::User(p) => p.public_key_pem(),
            UserOrInstance::Instance(p) => p.public_key_pem(),
        }
    }

    fn private_key_pem(&self) -> Option<String> {
        match self {
            UserOrInstance::User(p) => p.private_key_pem(),
            UserOrInstance::Instance(p) => p.private_key_pem(),
        }
    }

    fn inbox(&self) -> Url {
        unimplemented!()
    }
}
