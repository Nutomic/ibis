use super::{
    activities::comment::{
        create_or_update_comment::CreateOrUpdateComment,
        delete_comment::DeleteComment,
        undo_delete_comment::UndoDeleteComment,
    },
    objects::{
        article::ArticleWrapper,
        comment::{ApubComment, CommentWrapper},
        instance::InstanceWrapper,
        instance_collection::{ApubInstanceCollection, InstanceCollection},
        user::PersonWrapper,
    },
};
use crate::{
    activities::{
        accept::Accept,
        announce::AnnounceActivity,
        create_article::CreateArticle,
        follow::Follow,
        reject::RejectEdit,
        undo_follow::UndoFollow,
        update_local_article::UpdateLocalArticle,
        update_remote_article::UpdateRemoteArticle,
    },
    objects::{
        article::ApubArticle,
        articles_collection::{ApubArticleCollection, ArticleCollection},
        edits_collection::{ApubEditCollection, EditCollection},
        instance::ApubInstance,
        user::ApubUser,
    },
};
use activitypub_federation::{
    axum::{
        inbox::{ActivityData, receive_activity},
        json::FederationJson,
    },
    config::Data,
    protocol::context::WithContext,
    traits::{ActivityHandler, Actor, Collection, Object},
};
use axum::{
    Router,
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
};
use axum_macros::debug_handler;
use chrono::{DateTime, Utc};
use ibis_database::{
    common::{
        article::Article,
        comment::Comment,
        instance::Instance,
        newtypes::CommentId,
        user::Person,
    },
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

pub fn federation_routes() -> Router<()> {
    Router::new()
        .route("/", get(http_get_instance))
        .route("/user/:name", get(http_get_person))
        .route("/all_articles", get(http_get_all_articles))
        .route("/linked_instances", get(http_get_linked_instances))
        .route("/article/:title", get(http_get_article))
        .route("/article/:title/edits", get(http_get_article_edits))
        .route("/comment/:id", get(http_get_comment))
        .route("/inbox", post(http_post_inbox))
}

#[debug_handler]
async fn http_get_instance(
    context: Data<IbisContext>,
) -> BackendResult<FederationJson<WithContext<ApubInstance>>> {
    let local_instance: InstanceWrapper = Instance::read_local(&context)?.into();
    let json_instance = local_instance.into_json(&context).await?;
    Ok(FederationJson(WithContext::new_default(json_instance)))
}

#[debug_handler]
async fn http_get_person(
    Path(name): Path<String>,
    context: Data<IbisContext>,
) -> BackendResult<FederationJson<WithContext<ApubUser>>> {
    let person: PersonWrapper = Person::read_local_from_name(&name, &context)?.person.into();
    let json_person = person.into_json(&context).await?;
    Ok(FederationJson(WithContext::new_default(json_person)))
}

#[debug_handler]
async fn http_get_all_articles(
    context: Data<IbisContext>,
) -> BackendResult<FederationJson<WithContext<ApubArticleCollection>>> {
    let collection = ArticleCollection::read_local(&(), &context).await?;
    Ok(FederationJson(WithContext::new_default(collection)))
}

#[debug_handler]
async fn http_get_linked_instances(
    context: Data<IbisContext>,
) -> BackendResult<FederationJson<WithContext<ApubInstanceCollection>>> {
    let collection = InstanceCollection::read_local(&(), &context).await?;
    Ok(FederationJson(WithContext::new_default(collection)))
}

#[debug_handler]
async fn http_get_article(
    Path(title): Path<String>,
    context: Data<IbisContext>,
) -> BackendResult<FederationJson<WithContext<ApubArticle>>> {
    let article: ArticleWrapper = Article::read_view((&title, None), None, &context)?
        .article
        .into();
    let json = article.into_json(&context).await?;
    Ok(FederationJson(WithContext::new_default(json)))
}

#[debug_handler]
async fn http_get_article_edits(
    Path(title): Path<String>,
    context: Data<IbisContext>,
) -> BackendResult<FederationJson<WithContext<ApubEditCollection>>> {
    let article = Article::read_view((&title, None), None, &context)?;
    let json = EditCollection::read_local(&article.article, &context).await?;
    Ok(FederationJson(WithContext::new_default(json)))
}

#[debug_handler]
async fn http_get_comment(
    Path(id): Path<i32>,
    context: Data<IbisContext>,
) -> BackendResult<FederationJson<WithContext<ApubComment>>> {
    let comment: CommentWrapper = Comment::read(CommentId(id), &context)?.into();
    let json = comment.into_json(&context).await?;
    Ok(FederationJson(WithContext::new_default(json)))
}

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum InboxActivities {
    Follow(Follow),
    UndoFollow(UndoFollow),
    Accept(Accept),
    CreateArticle(CreateArticle),
    UpdateLocalArticle(UpdateLocalArticle),
    UpdateRemoteArticle(UpdateRemoteArticle),
    RejectEdit(RejectEdit),
    AnnounceActivity(AnnounceActivity),
    AnnouncableActivities(AnnouncableActivities),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum AnnouncableActivities {
    CreateOrUpdateComment(CreateOrUpdateComment),
    DeleteComment(DeleteComment),
    UndoDeleteComment(UndoDeleteComment),
}

#[debug_handler]
pub async fn http_post_inbox(
    context: Data<IbisContext>,
    activity_data: ActivityData,
) -> impl IntoResponse {
    receive_activity::<WithContext<InboxActivities>, UserOrInstance, _>(activity_data, &context)
        .await
}

#[derive(Clone, Debug)]
pub enum UserOrInstance {
    User(PersonWrapper),
    Instance(InstanceWrapper),
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
    Service,
}

#[async_trait::async_trait]
impl Object for UserOrInstance {
    type DataType = IbisContext;
    type Kind = PersonOrInstance;
    type Error = BackendError;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        Some(match self {
            UserOrInstance::User(p) => p.last_refreshed_at,
            UserOrInstance::Instance(p) => p.last_refreshed_at,
        })
    }

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, BackendError> {
        let person = PersonWrapper::read_from_id(object_id.clone(), data).await;
        Ok(match person {
            Ok(Some(o)) => Some(UserOrInstance::User(o)),
            _ => InstanceWrapper::read_from_id(object_id.clone(), data)
                .await?
                .map(UserOrInstance::Instance),
        })
    }

    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), BackendError> {
        match self {
            UserOrInstance::User(p) => p.delete(data).await,
            UserOrInstance::Instance(p) => p.delete(data).await,
        }
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, BackendError> {
        unimplemented!()
    }

    async fn verify(
        apub: &Self::Kind,
        expected_domain: &Url,
        data: &Data<Self::DataType>,
    ) -> Result<(), BackendError> {
        match apub {
            PersonOrInstance::Person(a) => PersonWrapper::verify(a, expected_domain, data).await,
            PersonOrInstance::Instance(a) => {
                InstanceWrapper::verify(a, expected_domain, data).await
            }
        }
    }

    async fn from_json(
        apub: Self::Kind,
        data: &Data<Self::DataType>,
    ) -> Result<Self, BackendError> {
        Ok(match apub {
            PersonOrInstance::Person(p) => {
                UserOrInstance::User(PersonWrapper::from_json(p, data).await?)
            }
            PersonOrInstance::Instance(p) => {
                UserOrInstance::Instance(InstanceWrapper::from_json(p, data).await?)
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
