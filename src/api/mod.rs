use crate::api::article::create_article;
use crate::api::article::{edit_article, fork_article, get_article};
use crate::api::instance::follow_instance;
use crate::api::instance::get_local_instance;
use crate::api::user::login_user;
use crate::api::user::register_user;
use crate::database::article::{ArticleView, DbArticle};
use crate::database::conflict::{ApiConflict, DbConflict};
use crate::database::edit::DbEdit;
use crate::database::instance::DbInstance;
use crate::database::MyDataHandle;
use crate::error::MyResult;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_macros::debug_handler;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use url::Url;

pub mod article;
pub mod instance;
pub mod user;

pub fn api_routes() -> Router {
    Router::new()
        .route(
            "/article",
            get(get_article).post(create_article).patch(edit_article),
        )
        .route("/article/fork", post(fork_article))
        .route("/edit_conflicts", get(edit_conflicts))
        .route("/resolve_instance", get(resolve_instance))
        .route("/resolve_article", get(resolve_article))
        .route("/instance", get(get_local_instance))
        .route("/instance/follow", post(follow_instance))
        .route("/search", get(search_article))
        .route("/user/register", post(register_user))
        .route("/user/login", post(login_user))
}

#[derive(Deserialize, Serialize)]
pub struct ResolveObject {
    pub id: Url,
}

/// Fetch a remote instance actor. This automatically synchronizes the remote articles collection to
/// the local instance, and allows for interactions such as following.
#[debug_handler]
async fn resolve_instance(
    Query(query): Query<ResolveObject>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<DbInstance>> {
    let instance: DbInstance = ObjectId::from(query.id).dereference(&data).await?;
    Ok(Json(instance))
}

/// Fetch a remote article, including edits collection. Allows viewing and editing. Note that new
/// article changes can only be received if we follow the instance, or if it is refetched manually.
#[debug_handler]
async fn resolve_article(
    Query(query): Query<ResolveObject>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<ArticleView>> {
    let article: DbArticle = ObjectId::from(query.id).dereference(&data).await?;
    let edits = DbEdit::read_for_article(&article, &data.db_connection)?;
    let latest_version = edits.last().unwrap().hash.clone();
    Ok(Json(ArticleView {
        article,
        edits,
        latest_version,
    }))
}

/// Get a list of all unresolved edit conflicts.
#[debug_handler]
async fn edit_conflicts(data: Data<MyDataHandle>) -> MyResult<Json<Vec<ApiConflict>>> {
    let conflicts = DbConflict::list(&data.db_connection)?;
    let conflicts: Vec<ApiConflict> = try_join_all(conflicts.into_iter().map(|c| {
        let data = data.reset_request_count();
        async move { c.to_api_conflict(&data).await }
    }))
    .await?
    .into_iter()
    .flatten()
    .collect();
    Ok(Json(conflicts))
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SearchArticleData {
    pub query: String,
}

/// Search articles for matching title or body text.
#[debug_handler]
async fn search_article(
    Query(query): Query<SearchArticleData>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<Vec<DbArticle>>> {
    let article = DbArticle::search(&query.query, &data.db_connection)?;
    Ok(Json(article))
}
