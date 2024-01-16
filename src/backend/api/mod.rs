use crate::backend::api::article::create_article;
use crate::backend::api::article::{edit_article, fork_article, get_article};
use crate::backend::api::instance::follow_instance;
use crate::backend::api::instance::get_local_instance;
use crate::backend::api::user::my_profile;
use crate::backend::api::user::register_user;
use crate::backend::api::user::validate;
use crate::backend::api::user::{login_user, logout_user};
use crate::backend::database::conflict::{ApiConflict, DbConflict};
use crate::backend::database::instance::DbInstance;
use crate::backend::database::MyDataHandle;
use crate::backend::error::MyResult;
use crate::common::DbEdit;
use crate::common::LocalUserView;
use crate::common::{ArticleView, DbArticle};
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{
    extract::TypedHeader,
    headers::authorization::{Authorization, Bearer},
    http::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    Extension,
};
use axum::{Json, Router};
use axum_macros::debug_handler;
use futures::future::try_join_all;
use log::warn;
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
        .route("/account/register", post(register_user))
        .route("/account/login", post(login_user))
        .route("/account/my_profile", get(my_profile))
        .route("/account/logout", get(logout_user))
        .route_layer(middleware::from_fn(auth))
}

async fn auth<B>(
    data: Data<MyDataHandle>,
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    if let Some(auth) = auth {
        let user = validate(auth.token(), &data).await.map_err(|e| {
            warn!("Failed to validate auth token: {e}");
            StatusCode::UNAUTHORIZED
        })?;
        request.extensions_mut().insert(user);
    }
    let response = next.run(request).await;
    Ok(response)
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
    // TODO: workaround because axum makes it hard to have multiple routes on /
    let id = format!("{}instance", query.id);
    let instance: DbInstance = ObjectId::parse(&id)?.dereference(&data).await?;
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
async fn edit_conflicts(
    Extension(user): Extension<LocalUserView>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<Vec<ApiConflict>>> {
    let conflicts = DbConflict::list(&user.local_user, &data.db_connection)?;
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
