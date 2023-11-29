use crate::database::MyDataHandle;
use crate::error::MyResult;
use crate::federation::activities::accept::Accept;
use crate::federation::activities::follow::Follow;
use crate::federation::objects::instance::{ApubInstance, DbInstance};

use activitypub_federation::axum::inbox::{receive_activity, ActivityData};
use activitypub_federation::axum::json::FederationJson;
use activitypub_federation::config::Data;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::Object;
use activitypub_federation::traits::{ActivityHandler, Collection};
use axum::extract::Path;

use crate::federation::activities::create_article::CreateArticle;
use crate::federation::activities::reject::RejectEdit;
use crate::federation::activities::update_local_article::UpdateLocalArticle;
use crate::federation::activities::update_remote_article::UpdateRemoteArticle;
use crate::federation::objects::article::ApubArticle;
use crate::federation::objects::articles_collection::{ArticleCollection, DbArticleCollection};
use crate::federation::objects::edits_collection::{ApubEditCollection, DbEditCollection};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use url::Url;

pub fn federation_routes() -> Router {
    Router::new()
        .route("/", get(http_get_instance))
        .route("/all_articles", get(http_get_all_articles))
        .route("/article/:title", get(http_get_article))
        .route("/article/:title/edits", get(http_get_article_edits))
        .route("/inbox", post(http_post_inbox))
}

#[debug_handler]
async fn http_get_instance(
    data: Data<MyDataHandle>,
) -> MyResult<FederationJson<WithContext<ApubInstance>>> {
    let db_instance = data.local_instance();
    let json_instance = db_instance.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(json_instance)))
}

#[debug_handler]
async fn http_get_all_articles(
    data: Data<MyDataHandle>,
) -> MyResult<FederationJson<WithContext<ArticleCollection>>> {
    let collection = DbArticleCollection::read_local(&data.local_instance(), &data).await?;
    Ok(FederationJson(WithContext::new_default(collection)))
}

#[debug_handler]
async fn http_get_article(
    Path(title): Path<String>,
    data: Data<MyDataHandle>,
) -> MyResult<FederationJson<WithContext<ApubArticle>>> {
    let article = {
        let lock = data.articles.lock().unwrap();
        lock.values().find(|a| a.title == title).unwrap().clone()
    };
    let json = article.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(json)))
}

#[debug_handler]
async fn http_get_article_edits(
    Path(title): Path<String>,
    data: Data<MyDataHandle>,
) -> MyResult<FederationJson<WithContext<ApubEditCollection>>> {
    let article = {
        let lock = data.articles.lock().unwrap();
        lock.values().find(|a| a.title == title).unwrap().clone()
    };
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
    receive_activity::<WithContext<InboxActivities>, DbInstance, MyDataHandle>(activity_data, &data)
        .await
}
