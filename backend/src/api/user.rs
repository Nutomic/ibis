use crate::database::user::{DbLocalUser, DbPerson, LocalUserView};
use crate::database::{read_jwt_secret, MyDataHandle};
use crate::error::MyResult;
use activitypub_federation::config::Data;
use anyhow::anyhow;
use axum::{Form, Json};
use axum_macros::debug_handler;
use bcrypt::verify;
use chrono::Utc;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::{decode, get_current_timestamp};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// local_user.id
    pub sub: String,
    /// hostname
    pub iss: String,
    /// Creation time as unix timestamp
    pub iat: i64,
    /// Expiration time
    pub exp: u64,
}

pub(in crate::api) fn generate_login_token(
    local_user: DbLocalUser,
    data: &Data<MyDataHandle>,
) -> MyResult<LoginResponse> {
    let hostname = data.domain().to_string();
    let claims = Claims {
        sub: local_user.id.to_string(),
        iss: hostname,
        iat: Utc::now().timestamp(),
        exp: get_current_timestamp(),
    };

    let secret = read_jwt_secret(data)?;
    let key = EncodingKey::from_secret(secret.as_bytes());
    let jwt = encode(&Header::default(), &claims, &key)?;
    Ok(LoginResponse { jwt })
}

pub async fn validate(jwt: &str, data: &Data<MyDataHandle>) -> MyResult<LocalUserView> {
    let validation = Validation::default();
    let secret = read_jwt_secret(data)?;
    let key = DecodingKey::from_secret(secret.as_bytes());
    let claims = decode::<Claims>(jwt, &key, &validation)?;
    DbPerson::read_local_from_id(claims.claims.sub.parse()?, data)
}

#[derive(Deserialize, Serialize)]
pub struct RegisterUserData {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    pub jwt: String,
}

#[debug_handler]
pub(in crate::api) async fn register_user(
    data: Data<MyDataHandle>,
    Form(form): Form<RegisterUserData>,
) -> MyResult<Json<LoginResponse>> {
    let user = DbPerson::create_local(form.username, form.password, &data)?;
    Ok(Json(generate_login_token(user.local_user, &data)?))
}

#[derive(Deserialize, Serialize)]
pub struct LoginUserData {
    pub username: String,
    pub password: String,
}

#[debug_handler]
pub(in crate::api) async fn login_user(
    data: Data<MyDataHandle>,
    Form(form): Form<LoginUserData>,
) -> MyResult<Json<LoginResponse>> {
    let user = DbPerson::read_local_from_name(&form.username, &data)?;
    let valid = verify(&form.password, &user.local_user.password_encrypted)?;
    if !valid {
        return Err(anyhow!("Invalid login").into());
    }
    Ok(Json(generate_login_token(user.local_user, &data)?))
}
