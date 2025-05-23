use crate::api::{
    article::{
        create_article,
        edit_article,
        fork_article,
        get_article,
        get_conflict,
        list_articles,
        protect_article,
        resolve_article,
        search_article,
    },
    comment::{create_comment, edit_comment},
    instance::{follow_instance, get_instance, resolve_instance},
    user::{get_user, login_user, logout_user, register::register_user},
};
use activitypub_federation::config::Data;
use anyhow::anyhow;
use article::{delete_conflict, follow_article, remove_article};
use axum::{
    Extension,
    Json,
    Router,
    extract::{Query, rejection::ExtensionRejection},
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use axum_macros::{FromRequestParts, debug_handler};
use comment::get_comment;
use http::StatusCode;
use ibis_api_client::article::GetEditList;
use ibis_database::{
    common::{
        article::{Edit, EditView},
        instance::{Instance, SiteView},
        user::{LocalUserView, Person},
    },
    error::BackendResult,
    impls::{IbisContext, edit::ViewEditParams},
};
use instance::{list_instance_views, update_instance};
use std::ops::Deref;
use user::{
    article_notif_mark_as_read,
    change_password,
    change_password_after_reset,
    count_notifications,
    get_user_follows,
    list_notifications,
    register::authenticate_with_oauth,
    request_reset_password,
    update_user_profile,
    verify_email,
};

mod article;
mod comment;
mod instance;
pub(super) mod user;

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
        .route("/article/remove", post(remove_article))
        .route("/article/follow", post(follow_article))
        .route("/edit/list", get(edit_list))
        .route("/conflict", get(get_conflict))
        .route("/conflict", delete(delete_conflict))
        .route("/comment", post(create_comment))
        .route("/comment", patch(edit_comment))
        .route("/comment", get(get_comment))
        .route("/instance", get(get_instance))
        .route("/instance", patch(update_instance))
        .route("/instance/follow", post(follow_instance))
        .route("/instance/resolve", get(resolve_instance))
        .route("/instance/list", get(list_instance_views))
        .route("/search", get(search_article))
        .route("/user", get(get_user))
        .route("/user/follows", get(get_user_follows))
        .route("/user/notifications/list", get(list_notifications))
        .route("/user/notifications/count", get(count_notifications))
        .route(
            "/user/notifications/mark_as_read",
            post(article_notif_mark_as_read),
        )
        .route("/account/register", post(register_user))
        .route("/account/login", post(login_user))
        .route("/account/logout", post(logout_user))
        .route("/account/update", post(update_user_profile))
        .route("/account/oauth/authenticate", post(authenticate_with_oauth))
        .route("/account/verify_email", post(verify_email))
        .route("/account/change_password", post(change_password))
        .route(
            "/account/request_reset_password",
            post(request_reset_password),
        )
        .route(
            "/account/change_password_after_reset",
            post(change_password_after_reset),
        )
        .route("/site", get(site_view))
}

pub fn check_is_admin(user: &LocalUserView) -> BackendResult<()> {
    if !user.local_user.admin {
        return Err(anyhow!("Only admin can perform this action").into());
    }
    Ok(())
}

#[debug_handler]
pub(crate) async fn site_view(
    context: Data<IbisContext>,
    user: UserExtOpt,
) -> BackendResult<Json<SiteView>> {
    let oauth_providers = context
        .conf
        .oauth_providers
        .clone()
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(SiteView {
        my_profile: user.inner(),
        config: context.conf.options.clone(),
        admin: Person::read_admin(&context)?,
        instance: Instance::read_local(&context)?,
        oauth_providers,
    }))
}

/// Get a list of all unresolved edit conflicts.
#[debug_handler]
pub async fn edit_list(
    Query(query): Query<GetEditList>,
    user: UserExtOpt,
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<EditView>>> {
    let params = if let Some(article_id) = query.article_id {
        ViewEditParams::ArticleId(article_id)
    } else if let Some(person_id) = query.person_id {
        ViewEditParams::PersonId(person_id)
    } else {
        return Err(anyhow!("Must provide article_id or person_id").into());
    };
    Ok(Json(Edit::list_views(params, &user.inner(), &context)?))
}

/// Trims the string param, and converts to None if it is empty
/// TODO: implement this as serde attribute
///       https://github.com/serde-rs/serde/issues/1425
fn empty_to_none(val: &mut Option<String>) {
    if let Some(val_) = val {
        *val_ = val_.trim().to_string();
        if val_.is_empty() {
            *val = None
        }
    }
}

#[derive(FromRequestParts)]
#[from_request(rejection(NotLoggedInError))]
pub struct UserExt {
    #[from_request(via(Extension))]
    local_user_view: LocalUserView,
}

#[derive(FromRequestParts)]
pub struct UserExtOpt {
    #[from_request(via(Extension))]
    local_user_view: Option<LocalUserView>,
}

impl UserExt {
    pub fn inner(self) -> LocalUserView {
        self.local_user_view
    }
}
impl UserExtOpt {
    pub fn inner(self) -> Option<LocalUserView> {
        self.local_user_view
    }
}
impl Deref for UserExt {
    type Target = LocalUserView;

    fn deref(&self) -> &Self::Target {
        &self.local_user_view
    }
}
impl Deref for UserExtOpt {
    type Target = Option<LocalUserView>;

    fn deref(&self) -> &Self::Target {
        &self.local_user_view
    }
}
impl From<ExtensionRejection> for NotLoggedInError {
    fn from(_: ExtensionRejection) -> Self {
        NotLoggedInError
    }
}
pub struct NotLoggedInError;

impl IntoResponse for NotLoggedInError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::FORBIDDEN, "Login required").into_response()
    }
}
