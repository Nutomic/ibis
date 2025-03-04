use super::{empty_to_none, UserExt};
use activitypub_federation::config::Data;
use anyhow::anyhow;
use axum::{extract::Query, Form, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, Expiration, SameSite};
use axum_macros::debug_handler;
use bcrypt::verify;
use chrono::Utc;
use ibis_api_client::{
    notifications::MarkAsReadParams,
    user::{GetUserParams, LoginUserParams, UpdateUserParams, VerifyEmailParams},
};
use ibis_database::{
    common::{
        instance::InstanceFollow,
        notifications::ApiNotification,
        user::{LocalUserView, Person},
        SuccessResponse, AUTH_COOKIE,
    },
    email::{send_validation_email, set_email_verified},
    error::{BackendError, BackendResult},
    impls::{
        notifications::Notification,
        read_jwt_secret,
        user::{LocalUserViewQuery, PersonUpdateForm},
        IbisContext,
    },
};
use ibis_federate::validate::{validate_display_name, validate_email};
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

pub(crate) fn generate_login_token(
    person: &Person,
    context: &Data<IbisContext>,
) -> BackendResult<String> {
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
    LocalUserView::read(
        LocalUserViewQuery::LocalNameOrEmail(&claims.claims.sub),
        context,
    )
}

#[debug_handler]
pub(crate) async fn login_user(
    context: Data<IbisContext>,
    jar: CookieJar,
    Form(params): Form<LoginUserParams>,
) -> BackendResult<(CookieJar, Json<LocalUserView>)> {
    let invalid_login: BackendError = anyhow!("Invalid login").into();
    let user = LocalUserView::read(
        LocalUserViewQuery::LocalNameOrEmail(&params.username_or_email),
        &context,
    )
    .map_err(|_| invalid_login)?;
    let valid = user
        .local_user
        .password_encrypted
        .as_ref()
        .and_then(|pw| verify(&params.password, pw).ok())
        .unwrap_or(false);
    if !valid {
        return Err(anyhow!("Invalid login").into());
    }
    let token = generate_login_token(&user.person, &context)?;
    let jar = jar.add(create_cookie(token, &context));
    Ok((jar, Json(user)))
}

pub(crate) fn create_cookie(jwt: String, context: &Data<IbisContext>) -> Cookie<'static> {
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
pub(crate) async fn logout_user(
    context: Data<IbisContext>,
    jar: CookieJar,
) -> BackendResult<(CookieJar, Json<SuccessResponse>)> {
    let jar = jar.remove(create_cookie(String::new(), &context));
    Ok((jar, Json(SuccessResponse::default())))
}

#[debug_handler]
pub(crate) async fn get_user(
    params: Query<GetUserParams>,
    context: Data<IbisContext>,
) -> BackendResult<Json<Person>> {
    Ok(Json(Person::read_from_name(
        &params.name,
        &params.domain,
        &context,
    )?))
}

#[debug_handler]
pub(crate) async fn get_user_follows(
    user: UserExt,
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<InstanceFollow>>> {
    Ok(Json(Person::read_following(user.person.id, &context)?))
}

#[debug_handler]
pub(crate) async fn update_user_profile(
    context: Data<IbisContext>,
    user: UserExt,
    Form(mut params): Form<UpdateUserParams>,
) -> BackendResult<Json<SuccessResponse>> {
    empty_to_none(&mut params.display_name);
    empty_to_none(&mut params.bio);
    empty_to_none(&mut params.email);
    validate_display_name(&params.display_name)?;
    let person_form = PersonUpdateForm {
        display_name: params.display_name,
        bio: params.bio,
    };
    // update, ignoring empty query errors
    Person::update_profile(&person_form, user.person.id, &context).ok();

    // send validation email, which stores the address and applies it to user once verified
    if let Some(email) = params.email {
        validate_email(&email)?;
        send_validation_email(&user, &email, &context).await?;
    }
    Ok(Json(SuccessResponse::default()))
}

#[debug_handler]
pub(crate) async fn list_notifications(
    user: UserExt,
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<ApiNotification>>> {
    Ok(Json(Notification::list(&user, &context).await?))
}

#[debug_handler]
pub(crate) async fn count_notifications(
    user: Option<UserExt>,
    context: Data<IbisContext>,
) -> BackendResult<Json<i64>> {
    if let Some(user) = user {
        Ok(Json(Notification::count(&user, &context)?))
    } else {
        Ok(Json(0))
    }
}

#[debug_handler]
pub(crate) async fn article_notif_mark_as_read(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<MarkAsReadParams>,
) -> BackendResult<Json<SuccessResponse>> {
    Notification::mark_as_read(params.id, &user, &context)?;
    Ok(Json(SuccessResponse::default()))
}

#[debug_handler]
pub(crate) async fn verify_email(
    context: Data<IbisContext>,
    Form(params): Form<VerifyEmailParams>,
) -> BackendResult<Json<SuccessResponse>> {
    set_email_verified(&params.token, &context)?;
    Ok(Json(SuccessResponse::default()))
}
