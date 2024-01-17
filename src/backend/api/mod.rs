use crate::backend::api::article::{create_article, resolve_article};
use crate::backend::api::article::{edit_article, fork_article, get_article};
use crate::backend::api::instance::get_local_instance;
use crate::backend::api::instance::{follow_instance, resolve_instance};
use crate::backend::api::user::register_user;
use crate::backend::api::user::validate;
use crate::backend::api::user::{login_user, logout_user};
use crate::backend::api::user::{my_profile, AUTH_COOKIE};
use crate::backend::database::conflict::{ApiConflict, DbConflict};
use crate::backend::database::MyDataHandle;
use crate::backend::error::MyResult;
use crate::common::DbArticle;
use crate::common::LocalUserView;
use activitypub_federation::config::Data;
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{
    http::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    Extension,
};
use axum::{Json, Router};
use axum_extra::extract::CookieJar;
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
        .route("/article/resolve", get(resolve_article))
        .route("/edit_conflicts", get(edit_conflicts))
        .route("/instance", get(get_local_instance))
        .route("/instance/follow", post(follow_instance))
        .route("/instance/resolve", get(resolve_instance))
        .route("/search", get(search_article))
        .route("/account/register", post(register_user))
        .route("/account/login", post(login_user))
        .route("/account/my_profile", get(my_profile))
        .route("/account/logout", get(logout_user))
        .route_layer(middleware::from_fn(auth))
}

async fn auth<B>(
    data: Data<MyDataHandle>,
    jar: CookieJar,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    if let Some(auth) = jar.get(AUTH_COOKIE) {
        let user = validate(auth.value(), &data).await.map_err(|e| {
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
