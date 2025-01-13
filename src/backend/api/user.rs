use super::{check_is_admin, empty_to_none};
use crate::{
    backend::{
        database::{conflict::DbConflict, read_jwt_secret, IbisData},
        error::MyResult,
    },
    common::{
        DbArticle,
        DbPerson,
        GetUserForm,
        LocalUserView,
        LoginUserForm,
        Notification,
        RegisterUserForm,
        SuccessResponse,
        UpdateUserForm,
        AUTH_COOKIE,
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
    decode,
    encode,
    get_current_timestamp,
    DecodingKey,
    EncodingKey,
    Header,
    Validation,
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

fn generate_login_token(person: &DbPerson, data: &Data<IbisData>) -> MyResult<String> {
    let hostname = data.domain().to_string();
    let claims = Claims {
        sub: person.username.clone(),
        iss: hostname,
        iat: Utc::now().timestamp(),
        exp: get_current_timestamp() + 60 * 60 * 24 * 365,
    };

    let secret = read_jwt_secret(data)?;
    let key = EncodingKey::from_secret(secret.as_bytes());
    let jwt = encode(&Header::default(), &claims, &key)?;
    Ok(jwt)
}

pub async fn validate(jwt: &str, data: &Data<IbisData>) -> MyResult<LocalUserView> {
    let validation = Validation::default();
    let secret = read_jwt_secret(data)?;
    let key = DecodingKey::from_secret(secret.as_bytes());
    let claims = decode::<Claims>(jwt, &key, &validation)?;
    DbPerson::read_local_from_name(&claims.claims.sub, data)
}

#[debug_handler]
pub(in crate::backend::api) async fn register_user(
    data: Data<IbisData>,
    jar: CookieJar,
    Form(form): Form<RegisterUserForm>,
) -> MyResult<(CookieJar, Json<LocalUserView>)> {
    if !data.config.options.registration_open {
        return Err(anyhow!("Registration is closed").into());
    }
    let user = DbPerson::create_local(form.username, form.password, false, &data)?;
    let token = generate_login_token(&user.person, &data)?;
    let jar = jar.add(create_cookie(token, &data));
    Ok((jar, Json(user)))
}

#[debug_handler]
pub(in crate::backend::api) async fn login_user(
    data: Data<IbisData>,
    jar: CookieJar,
    Form(form): Form<LoginUserForm>,
) -> MyResult<(CookieJar, Json<LocalUserView>)> {
    let user = DbPerson::read_local_from_name(&form.username, &data)?;
    let valid = verify(&form.password, &user.local_user.password_encrypted)?;
    if !valid {
        return Err(anyhow!("Invalid login").into());
    }
    let token = generate_login_token(&user.person, &data)?;
    let jar = jar.add(create_cookie(token, &data));
    Ok((jar, Json(user)))
}

fn create_cookie(jwt: String, data: &Data<IbisData>) -> Cookie<'static> {
    let mut cookie = Cookie::build((AUTH_COOKIE, jwt));

    // Must not set cookie domain on localhost
    // https://stackoverflow.com/a/1188145
    let domain = data.domain().to_string();
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
    data: Data<IbisData>,
    jar: CookieJar,
) -> MyResult<(CookieJar, Json<SuccessResponse>)> {
    let jar = jar.remove(create_cookie(String::new(), &data));
    Ok((jar, Json(SuccessResponse::default())))
}

#[debug_handler]
pub(in crate::backend::api) async fn get_user(
    params: Query<GetUserForm>,
    data: Data<IbisData>,
) -> MyResult<Json<DbPerson>> {
    Ok(Json(DbPerson::read_from_name(
        &params.name,
        &params.domain,
        &data,
    )?))
}

#[debug_handler]
pub(in crate::backend::api) async fn update_user_profile(
    data: Data<IbisData>,
    Form(mut params): Form<UpdateUserForm>,
) -> MyResult<Json<SuccessResponse>> {
    empty_to_none(&mut params.display_name);
    empty_to_none(&mut params.bio);
    DbPerson::update_profile(&params, &data)?;
    Ok(Json(SuccessResponse::default()))
}

#[debug_handler]
pub(crate) async fn list_notifications(
    Extension(user): Extension<LocalUserView>,
    data: Data<IbisData>,
) -> MyResult<Json<Vec<Notification>>> {
    let conflicts = DbConflict::list(&user.person, &data)?;
    let conflicts: Vec<_> = try_join_all(conflicts.into_iter().map(|c| {
        let data = data.reset_request_count();
        async move { c.to_api_conflict(&data).await }
    }))
    .await?;
    let mut notifications: Vec<_> = conflicts
        .into_iter()
        .flatten()
        .map(Notification::EditConflict)
        .collect();

    if check_is_admin(&user).is_ok() {
        let articles = DbArticle::list_approval_required(&data)?;
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
    Extension(user): Extension<LocalUserView>,
    data: Data<IbisData>,
) -> MyResult<Json<usize>> {
    let mut count = 0;
    let conflicts = DbConflict::list(&user.person, &data)?;
    count += conflicts.len();
    if check_is_admin(&user).is_ok() {
        let articles = DbArticle::list_approval_required(&data)?;
        count += articles.len();
    }

    Ok(Json(count))
}
