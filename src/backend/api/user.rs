use crate::backend::database::{read_jwt_secret, MyDataHandle};
use crate::backend::error::MyResult;
use crate::common::{DbLocalUser, DbPerson, LocalUserView, LoginUserData, RegisterUserData};
use activitypub_federation::config::Data;
use anyhow::anyhow;
use axum::{Form, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, Expiration, SameSite};
use axum_macros::debug_handler;
use bcrypt::verify;
use chrono::Utc;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::{decode, get_current_timestamp};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

pub static AUTH_COOKIE: &str = "auth";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// local_user.id
    pub sub: String,
    /// hostname
    pub iss: String,
    /// Creation time as unix timestamp
    pub iat: i64,
    /// Expiration time
    pub exp: u64,
}

fn generate_login_token(local_user: &DbLocalUser, data: &Data<MyDataHandle>) -> MyResult<String> {
    let hostname = data.domain().to_string();
    let claims = Claims {
        sub: local_user.id.to_string(),
        iss: hostname,
        iat: Utc::now().timestamp(),
        exp: get_current_timestamp() + 60 * 60 * 24 * 365,
    };

    let secret = read_jwt_secret(data)?;
    let key = EncodingKey::from_secret(secret.as_bytes());
    let jwt = encode(&Header::default(), &claims, &key)?;
    Ok(jwt)
}

pub async fn validate(jwt: &str, data: &Data<MyDataHandle>) -> MyResult<LocalUserView> {
    let validation = Validation::default();
    let secret = read_jwt_secret(data)?;
    let key = DecodingKey::from_secret(secret.as_bytes());
    let claims = decode::<Claims>(jwt, &key, &validation)?;
    DbPerson::read_local_from_id(claims.claims.sub.parse()?, data)
}

#[debug_handler]
pub(in crate::backend::api) async fn register_user(
    data: Data<MyDataHandle>,
    jar: CookieJar,
    Form(form): Form<RegisterUserData>,
) -> MyResult<(CookieJar, Json<LocalUserView>)> {
    if !data.config.registration_open {
        return Err(anyhow!("Registration is closed").into());
    }
    let user = DbPerson::create_local(form.username, form.password, false, &data)?;
    let token = generate_login_token(&user.local_user, &data)?;
    let jar = jar.add(create_cookie(token, &data));
    Ok((jar, Json(user)))
}

#[debug_handler]
pub(in crate::backend::api) async fn login_user(
    data: Data<MyDataHandle>,
    jar: CookieJar,
    Form(form): Form<LoginUserData>,
) -> MyResult<(CookieJar, Json<LocalUserView>)> {
    let user = DbPerson::read_local_from_name(&form.username, &data)?;
    let valid = verify(&form.password, &user.local_user.password_encrypted)?;
    if !valid {
        return Err(anyhow!("Invalid login").into());
    }
    let token = generate_login_token(&user.local_user, &data)?;
    let jar = jar.add(create_cookie(token, &data));
    Ok((jar, Json(user)))
}

fn create_cookie(jwt: String, data: &Data<MyDataHandle>) -> Cookie<'static> {
    let mut domain = data.domain().to_string();
    // remove port from domain
    if domain.contains(':') {
        domain = domain.split(':').collect::<Vec<_>>()[0].to_string();
    }
    Cookie::build(AUTH_COOKIE, jwt)
        .domain(domain)
        .same_site(SameSite::Strict)
        .path("/")
        .http_only(true)
        .secure(true)
        .expires(Expiration::DateTime(
            OffsetDateTime::now_utc() + Duration::weeks(52),
        ))
        .finish()
}

#[debug_handler]
pub(in crate::backend::api) async fn my_profile(
    data: Data<MyDataHandle>,
    jar: CookieJar,
) -> MyResult<Json<LocalUserView>> {
    let jwt = jar.get(AUTH_COOKIE).map(|c| c.value());
    if let Some(jwt) = jwt {
        Ok(Json(validate(jwt, &data).await?))
    } else {
        Err(anyhow!("invalid/missing auth").into())
    }
}

#[debug_handler]
pub(in crate::backend::api) async fn logout_user(
    data: Data<MyDataHandle>,
    jar: CookieJar,
) -> MyResult<CookieJar> {
    let jar = jar.remove(create_cookie(String::new(), &data));
    Ok(jar)
}
