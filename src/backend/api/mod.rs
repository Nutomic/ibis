use super::database::edit::ViewEditParams;
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
            user::{get_user, login_user, logout_user, register_user},
        },
        database::IbisData,
        error::MyResult,
    },
    common::{DbEdit, EditView, GetEditList, LocalUserView, SiteView},
};
use activitypub_federation::config::Data;
use anyhow::anyhow;
use article::{approve_article, delete_conflict};
use axum::{
    extract::Query,
    routing::{delete, get, post},
    Extension,
    Json,
    Router,
};
use axum_macros::debug_handler;
use instance::list_remote_instances;
use user::{count_notifications, list_notifications, update_user_profile};

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
        .route("/article/approve", post(approve_article))
        .route("/edit/list", get(edit_list))
        .route("/conflict", delete(delete_conflict))
        .route("/instance", get(get_instance))
        .route("/instance/follow", post(follow_instance))
        .route("/instance/resolve", get(resolve_instance))
        .route("/instance/list", get(list_remote_instances))
        .route("/search", get(search_article))
        .route("/user", get(get_user))
        .route("/user/notifications/list", get(list_notifications))
        .route("/user/notifications/count", get(count_notifications))
        .route("/account/register", post(register_user))
        .route("/account/login", post(login_user))
        .route("/account/logout", post(logout_user))
        .route("/account/update", post(update_user_profile))
        .route("/site", get(site_view))
}

fn check_is_admin(user: &LocalUserView) -> MyResult<()> {
    if !user.local_user.admin {
        return Err(anyhow!("Only admin can perform this action").into());
    }
    Ok(())
}

#[debug_handler]
pub(in crate::backend::api) async fn site_view(
    data: Data<IbisData>,
    user: Option<Extension<LocalUserView>>,
) -> MyResult<Json<SiteView>> {
    Ok(Json(SiteView {
        my_profile: user.map(|u| u.0),
        config: data.config.options.clone(),
    }))
}

/// Get a list of all unresolved edit conflicts.
#[debug_handler]
pub async fn edit_list(
    Query(query): Query<GetEditList>,
    data: Data<IbisData>,
) -> MyResult<Json<Vec<EditView>>> {
    let params = if let Some(article_id) = query.article_id {
        ViewEditParams::ArticleId(article_id)
    } else if let Some(person_id) = query.person_id {
        ViewEditParams::PersonId(person_id)
    } else {
        return Err(anyhow!("Must provide article_id or person_id").into());
    };
    Ok(Json(DbEdit::view(params, &data)?))
}

/// Trims the string param, and converts to None if it is empty
fn empty_to_none(val: &mut Option<String>) {
    (*val) = val.as_ref().map(|s| s.trim().to_owned());
}
