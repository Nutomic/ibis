use crate::{
    backend::{
        api::{
            article::{
                create_article,
                edit_article,
                fork_article,
                get_article,
                list_articles,
                protect_article,
                resolve_article,
                search_article,
            },
            instance::{follow_instance, get_instance, resolve_instance},
            user::{
                get_user,
                login_user,
                logout_user,
                my_profile,
                register_user,
                validate,
                AUTH_COOKIE,
            },
        },
        database::{conflict::DbConflict, IbisData},
        error::MyResult,
    },
    common::{ApiConflict, LocalUserView},
};
use activitypub_federation::config::Data;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Extension,
    Json,
    Router,
};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use futures::future::try_join_all;
use instance::list_remote_instances;

pub mod article;
pub mod instance;
pub mod user;

pub fn api_routes() -> Router<()> {
    Router::new()
        .route(
            "/article",
            get(get_article).post(create_article).patch(edit_article),
        )
        .route("/article/list", get(list_articles))
        .route("/article/fork", post(fork_article))
        .route("/article/resolve", get(resolve_article))
        .route("/article/protect", post(protect_article))
        .route("/edit_conflicts", get(edit_conflicts))
        .route("/instance", get(get_instance))
        .route("/instance/follow", post(follow_instance))
        .route("/instance/resolve", get(resolve_instance))
        .route("/instance/list", get(list_remote_instances))
        .route("/search", get(search_article))
        .route("/user", get(get_user))
        .route("/account/register", post(register_user))
        .route("/account/login", post(login_user))
        .route("/account/my_profile", get(my_profile))
        .route("/account/logout", get(logout_user))
        .route_layer(middleware::from_fn(auth))
}

async fn auth(
    data: Data<IbisData>,
    jar: CookieJar,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(auth) = jar.get(AUTH_COOKIE) {
        if let Ok(user) = validate(auth.value(), &data).await {
            request.extensions_mut().insert(user);
        }
    }
    let response = next.run(request).await;
    Ok(response)
}

/// Get a list of all unresolved edit conflicts.
#[debug_handler]
async fn edit_conflicts(
    Extension(user): Extension<LocalUserView>,
    data: Data<IbisData>,
) -> MyResult<Json<Vec<ApiConflict>>> {
    let conflicts = DbConflict::list(&user.local_user, &data)?;
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
