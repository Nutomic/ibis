use crate::{
    activities::{
        announce::AnnounceActivity,
        article::{
            create_article::CreateArticle,
            edit_article::EditArticle,
            remove_article::RemoveArticle,
            undo_remove_article::UndoRemoveArticle,
            update_article::UpdateArticle,
        },
        comment::{
            create_or_update_comment::CreateOrUpdateComment,
            delete_comment::DeleteComment,
            undo_delete_comment::UndoDeleteComment,
        },
        following::{accept::Accept, follow::Follow, undo_follow::UndoFollow},
        reject::RejectEdit,
    },
    collections::{
        articles_collection::ArticleCollection,
        edits_collection::EditCollection,
        empty_outbox::EmptyOutbox,
        instance_collection::InstanceCollection,
        instance_follower::InstanceFollower,
    },
    objects::{
        article::ArticleWrapper,
        comment::CommentWrapper,
        instance::InstanceWrapper,
        user::PersonWrapper,
    },
};
use activitypub_federation::{
    axum::{
        inbox::{ActivityData, receive_activity},
        json::FederationJson,
    },
    config::Data,
    protocol::context::WithContext,
    traits::{ActivityHandler, Collection, Object},
};
use axum::{
    Router,
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
};
use axum_macros::debug_handler;
use either::Either;
use ibis_database::{
    common::{
        article::Article,
        comment::Comment,
        instance::Instance,
        newtypes::CommentId,
        user::Person,
    },
    error::BackendResult,
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

pub fn federation_routes() -> Router<()> {
    Router::new()
        .route("/", get(http_get_instance))
        .route("/outbox", get(http_get_instance_outbox))
        .route("/followers", get(http_get_instance_followers))
        .route("/user/:name", get(http_get_person))
        .route("/user/:name/outbox", get(http_get_person_outbox))
        .route("/all_articles", get(http_get_all_articles))
        .route("/linked_instances", get(http_get_linked_instances))
        .route("/article/:title", get(http_get_article))
        .route("/article/:title/edits", get(http_get_article_edits))
        .route("/comment/:id", get(http_get_comment))
        .route("/inbox", post(http_post_inbox))
}

#[debug_handler]
async fn http_get_instance(context: Data<IbisContext>) -> BackendResult<impl IntoResponse> {
    let local_instance: InstanceWrapper = Instance::read_local(&context)?.into();
    let json_instance = local_instance.into_json(&context).await?;
    Ok(FederationJson(WithContext::new_default(json_instance)))
}

#[debug_handler]
async fn http_get_instance_outbox(context: Data<IbisContext>) -> BackendResult<impl IntoResponse> {
    let articles = ArticleCollection::read_local(&(), &context).await?;
    Ok(FederationJson(WithContext::new_default(articles)))
}

#[debug_handler]
async fn http_get_instance_followers(
    context: Data<IbisContext>,
) -> BackendResult<impl IntoResponse> {
    let followers = InstanceFollower::read_local(&(), &context).await?;
    Ok(FederationJson(WithContext::new_default(followers)))
}

#[debug_handler]
async fn http_get_person(
    Path(name): Path<String>,
    context: Data<IbisContext>,
) -> BackendResult<impl IntoResponse> {
    let person: PersonWrapper = Person::read_from_name(&name, &None, &context)?.into();
    let json_person = person.into_json(&context).await?;
    Ok(FederationJson(WithContext::new_default(json_person)))
}

#[debug_handler]
async fn http_get_person_outbox(
    Path(name): Path<String>,
    context: Data<IbisContext>,
) -> BackendResult<impl IntoResponse> {
    let person = Person::read_from_name(&name, &None, &context)?;
    let outbox = EmptyOutbox::new(format!("{}/outbox", person.ap_id));
    Ok(FederationJson(WithContext::new_default(outbox)))
}

#[debug_handler]
async fn http_get_all_articles(context: Data<IbisContext>) -> BackendResult<impl IntoResponse> {
    let collection = ArticleCollection::read_local(&(), &context).await?;
    Ok(FederationJson(WithContext::new_default(collection)))
}

#[debug_handler]
async fn http_get_linked_instances(context: Data<IbisContext>) -> BackendResult<impl IntoResponse> {
    let collection = InstanceCollection::read_local(&(), &context).await?;
    Ok(FederationJson(WithContext::new_default(collection)))
}

#[debug_handler]
async fn http_get_article(
    Path(title): Path<String>,
    context: Data<IbisContext>,
) -> BackendResult<impl IntoResponse> {
    let title = title.replace("_", " ");
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
) -> BackendResult<impl IntoResponse> {
    let title = title.replace("_", " ");
    let article = Article::read_view((&title, None), None, &context)?;
    let json = EditCollection::read_local(&article.article, &context).await?;
    Ok(FederationJson(WithContext::new_default(json)))
}

#[debug_handler]
async fn http_get_comment(
    Path(id): Path<i32>,
    context: Data<IbisContext>,
) -> BackendResult<impl IntoResponse> {
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
    RejectEdit(RejectEdit),
    RemoveArticle(RemoveArticle),
    UndoRemoveArticle(UndoRemoveArticle),
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
    EditArticle(EditArticle),
    UpdateArticle(UpdateArticle),
}

#[debug_handler]
pub async fn http_post_inbox(
    context: Data<IbisContext>,
    activity_data: ActivityData,
) -> impl IntoResponse {
    receive_activity::<WithContext<InboxActivities>, Either<PersonWrapper, InstanceWrapper>, _>(
        activity_data,
        &context,
    )
    .await
}
