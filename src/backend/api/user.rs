use super::{check_is_admin, empty_to_none};
use crate::{
    backend::{
        database::{conflict::DbConflict, read_jwt_secret, IbisContext},
        utils::{
            error::BackendResult,
            validate::{validate_display_name, validate_user_name},
        },
    },
    common::{
        article::DbArticle,
        user::{
            DbPerson, GetUserParams, LocalUserView, LoginUserParams, RegisterUserParams,
            UpdateUserParams,
        },
        Notification, SuccessResponse, AUTH_COOKIE,
    },
};
use activitypub_federation::config::Data;
use anyhow::anyhow;
use axum::{extract::Query, Extension, Form, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, Expiration, SameSite};
use axum_macros::debug_handler;
use bcrypt::verify;
use chrono::Utc;
use futures::future::try_join_all;
use jsonwebtoken::{
    decode, encode, get_current_timestamp, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// person.username
    pub sub: String,
    /// hostname
    pub iss: String,
    /// Creation time as unix timestamp
    pub iat: i64,
    /// Expiration time
    pub exp: u64,
}

fn generate_login_token(person: &DbPerson, context: &Data<IbisContext>) -> BackendResult<String> {
    let hostname = context.domain().to_string();
    let claims = Claims {
        sub: person.username.clone(),
        iss: hostname,
        iat: Utc::now().timestamp(),
        exp: get_current_timestamp() + 60 * 60 * 24 * 365,
    };

    let secret = read_jwt_secret(context)?;
    let key = EncodingKey::from_secret(secret.as_bytes());
    let jwt = encode(&Header::default(), &claims, &key)?;
    Ok(jwt)
}

pub async fn validate(jwt: &str, context: &IbisContext) -> BackendResult<LocalUserView> {
    let validation = Validation::default();
    let secret = read_jwt_secret(context)?;
    let key = DecodingKey::from_secret(secret.as_bytes());
    let claims = decode::<Claims>(jwt, &key, &validation)?;
    DbPerson::read_local_from_name(&claims.claims.sub, context)
}

#[debug_handler]
pub(in crate::backend::api) async fn register_user(
    context: Data<IbisContext>,
    jar: CookieJar,
    Form(params): Form<RegisterUserParams>,
) -> BackendResult<(CookieJar, Json<LocalUserView>)> {
    if !context.config.options.registration_open {
        return Err(anyhow!("Registration is closed").into());
    }
    validate_user_name(&params.username)?;
    let user = DbPerson::create_local(params.username, params.password, false, &context)?;
    let token = generate_login_token(&user.person, &context)?;
    let jar = jar.add(create_cookie(token, &context));
    Ok((jar, Json(user)))
}

#[debug_handler]
pub(in crate::backend::api) async fn login_user(
    context: Data<IbisContext>,
    jar: CookieJar,
    Form(params): Form<LoginUserParams>,
) -> BackendResult<(CookieJar, Json<LocalUserView>)> {
    let user = DbPerson::read_local_from_name(&params.username, &context)?;
    let valid = verify(&params.password, &user.local_user.password_encrypted)?;
    if !valid {
        return Err(anyhow!("Invalid login").into());
    }
    let token = generate_login_token(&user.person, &context)?;
    let jar = jar.add(create_cookie(token, &context));
    Ok((jar, Json(user)))
}

fn create_cookie(jwt: String, context: &Data<IbisContext>) -> Cookie<'static> {
    let mut cookie = Cookie::build((AUTH_COOKIE, jwt));

    // Must not set cookie domain on localhost
    // https://stackoverflow.com/a/1188145
    let domain = context.domain().to_string();
    if !domain.starts_with("localhost") && !domain.starts_with("127.0.0.1") {
        cookie = cookie.domain(domain);
    }
    cookie
        .same_site(SameSite::Strict)
        .path("/")
        .http_only(true)
        .secure(!cfg!(debug_assertions))
        .expires(Expiration::DateTime(
            OffsetDateTime::now_utc() + Duration::weeks(52),
        ))
        .build()
}

#[debug_handler]
pub(in crate::backend::api) async fn logout_user(
    context: Data<IbisContext>,
    jar: CookieJar,
) -> BackendResult<(CookieJar, Json<SuccessResponse>)> {
    let jar = jar.remove(create_cookie(String::new(), &context));
    Ok((jar, Json(SuccessResponse::default())))
}

#[debug_handler]
pub(in crate::backend::api) async fn get_user(
    params: Query<GetUserParams>,
    context: Data<IbisContext>,
) -> BackendResult<Json<DbPerson>> {
    Ok(Json(DbPerson::read_from_name(
        &params.name,
        &params.domain,
        &context,
    )?))
}

#[debug_handler]
pub(in crate::backend::api) async fn update_user_profile(
    context: Data<IbisContext>,
    Form(mut params): Form<UpdateUserParams>,
) -> BackendResult<Json<SuccessResponse>> {
    empty_to_none(&mut params.display_name);
    empty_to_none(&mut params.bio);
    validate_display_name(&params.display_name)?;
    DbPerson::update_profile(&params, &context)?;
    Ok(Json(SuccessResponse::default()))
}

#[debug_handler]
pub(crate) async fn list_notifications(
    Extension(user): Extension<LocalUserView>,
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<Notification>>> {
    let conflicts = DbConflict::list(user.person.id, &context)?;
    let conflicts: Vec<_> = try_join_all(conflicts.into_iter().map(|c| {
        let data = context.reset_request_count();
        async move { c.to_api_conflict(&data).await }
    }))
    .await?;
    let mut notifications: Vec<_> = conflicts
        .into_iter()
        .flatten()
        .map(Notification::EditConflict)
        .collect();

    if check_is_admin(&user).is_ok() {
        let articles = DbArticle::list_approval_required(&context)?;
        notifications.extend(
            articles
                .into_iter()
                .map(Notification::ArticleApprovalRequired),
        )
    }
    notifications.sort_by(|a, b| a.published().cmp(b.published()));

    Ok(Json(notifications))
}

#[debug_handler]
pub(crate) async fn count_notifications(
    user: Option<Extension<LocalUserView>>,
    context: Data<IbisContext>,
) -> BackendResult<Json<usize>> {
    if let Some(user) = user {
        let mut count = 0;
        let conflicts = DbConflict::list(user.person.id, &context)?;
        count += conflicts.len();
        if check_is_admin(&user).is_ok() {
            let articles = DbArticle::list_approval_required(&context)?;
            count += articles.len();
        }

        Ok(Json(count))
    } else {
        Ok(Json(0))
    }
}
