use crate::api::{empty_to_none, user::add_login_cookie};
use activitypub_federation::config::Data;
use anyhow::anyhow;
use axum::{Form, Json};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use ibis_api_client::user::{
    AuthenticateWithOauth,
    OAuthTokenResponse,
    RegisterUserParams,
    RegistrationResponse,
};
use ibis_database::{
    common::user::{LocalUser, LocalUserView},
    config::OAuthProvider,
    email::verification::send_verification_email,
    error::{BackendError, BackendResult},
    impls::{
        IbisContext,
        user::{LocalUserViewQuery, OAuthAccount, OAuthAccountInsertForm},
    },
};
use ibis_federate::validate::{validate_email, validate_user_name};
use log::debug;
use serde::{Deserialize, Serialize};

type RegisterReturnType = BackendResult<(CookieJar, Json<RegistrationResponse>)>;

#[debug_handler]
pub async fn register_user(
    context: Data<IbisContext>,
    jar: CookieJar,
    Form(mut params): Form<RegisterUserParams>,
) -> RegisterReturnType {
    empty_to_none(&mut params.email);
    if !context.conf.options.registration_open {
        return Err(anyhow!("Registration is closed").into());
    }

    validate_new_password(&params.password, &params.confirm_password)?;

    if context.conf.options.email_required && params.email.is_none() {
        return Err(anyhow!("Email required").into());
    }

    check_new_user(&params.username, params.email.as_deref(), &context)?;

    // dont pass the email here, it needs to be validated first
    let user = LocalUserView::create(
        params.username,
        Some(params.password),
        false,
        None,
        &context,
    )?;

    if let Some(email) = &params.email {
        send_verification_email(&user.local_user, email, &context).await?;
    }

    register_return(user, jar, context.conf.options.email_required, &context)
}

#[debug_handler]
pub async fn authenticate_with_oauth(
    context: Data<IbisContext>,
    jar: CookieJar,
    Form(params): Form<AuthenticateWithOauth>,
) -> RegisterReturnType {
    let oauth_invalid_err: BackendError = anyhow!("Oauth Authorization is invalid").into();
    // validate inputs
    if params.code.is_empty() || params.code.len() > 300 {
        return Err(oauth_invalid_err);
    }

    // validate the redirect_uri
    let redirect_uri = &params.redirect_uri;
    if redirect_uri.host_str().unwrap_or("").is_empty()
        || !redirect_uri
            .path()
            .eq(&String::from("/account/oauth_callback"))
        || !redirect_uri.query().unwrap_or("").is_empty()
    {
        return Err(oauth_invalid_err);
    }

    // Fetch the OAUTH providers
    let oauth_provider = context
        .conf
        .oauth_providers
        .iter()
        .find(|provider| provider.issuer == params.oauth_issuer)
        .ok_or(oauth_invalid_err)?;

    let token_response = oauth_request_access_token(
        oauth_provider,
        &params.code,
        redirect_uri.as_str(),
        &context,
    )
    .await?;

    let user_info = oauth_get_user_info(
        oauth_provider,
        token_response.access_token.as_str(),
        &context,
    )
    .await?;
    let oauth_user_id = user_info.sub;
    let email = user_info.email;

    // Lookup user by oauth_user_id
    let mut local_user_view = LocalUserView::read(
        LocalUserViewQuery::Oauth(params.oauth_issuer.into(), &oauth_user_id),
        &context,
    );

    let user = if let Ok(user_view) = local_user_view {
        // user found by oauth_user_id => Login user
        user_view
    } else {
        // user has never previously registered using oauth

        // Lookup user by OAUTH email and link accounts
        local_user_view = LocalUserView::read(LocalUserViewQuery::Email(&email), &context);

        if let Ok(user) = local_user_view {
            // user found by email => link and login

            let oauth_account_form = OAuthAccountInsertForm {
                local_user_id: user.local_user.id,
                oauth_issuer_url: oauth_provider.issuer.clone().into(),
                oauth_user_id,
            };
            OAuthAccount::create(&oauth_account_form, &context)?;

            user
        } else {
            // No user was found by email => Register as new user

            let username = params
                .username
                .ok_or(anyhow!("Username is required to register new account"))?;

            check_new_user(&username, Some(&email), &context)?;
            let user = LocalUserView::create(username, None, false, Some(email), &context)?;

            // Create the oauth account
            let oauth_account_form = OAuthAccountInsertForm {
                local_user_id: user.local_user.id,
                oauth_issuer_url: oauth_provider.issuer.clone().into(),
                oauth_user_id,
            };
            OAuthAccount::create(&oauth_account_form, &context)?;

            user
        }
    };

    // dont require any email validation for oauth
    register_return(user, jar, false, &context)
}

/// Request an Access Token from the OAUTH provider
async fn oauth_request_access_token(
    oauth_provider: &OAuthProvider,
    code: &str,
    redirect_uri: &str,
    context: &IbisContext,
) -> BackendResult<OAuthTokenResponse> {
    let form = [
        ("client_id", &*oauth_provider.client_id),
        ("client_secret", &*oauth_provider.client_secret),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
    ];

    let response = context
        .client
        .post(oauth_provider.token_endpoint.as_str())
        .header("Accept", "application/json")
        .form(&form[..])
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    debug!("Oauth request access token response: status {status}, text {text}");

    Ok(serde_json::from_str(&text)?)
}

/// Request the user info from the OAUTH provider
async fn oauth_get_user_info(
    oauth_provider: &OAuthProvider,
    access_token: &str,
    context: &IbisContext,
) -> BackendResult<OauthUserInfo> {
    let response = context
        .client
        .get(oauth_provider.userinfo_endpoint.as_str())
        .header("Accept", "application/json")
        .bearer_auth(access_token)
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    debug!("Oauth get user info response: status {status}, text {text}");

    Ok(serde_json::from_str(&text)?)
}

#[derive(Serialize, Deserialize)]
struct OauthUserInfo {
    sub: String,
    email: String,
}

fn check_new_user(username: &str, email: Option<&str>, context: &IbisContext) -> BackendResult<()> {
    validate_user_name(username)?;
    LocalUser::check_username_taken(username, context)?;
    if let Some(email) = email {
        validate_email(email)?;
        LocalUser::check_email_taken(email, context)?;
    }
    Ok(())
}

fn register_return(
    user: LocalUserView,
    mut jar: CookieJar,
    email_verification_required: bool,
    context: &Data<IbisContext>,
) -> RegisterReturnType {
    if !email_verification_required {
        jar = add_login_cookie(&user.person, jar, context)?;
    }

    Ok((
        jar,
        Json(RegistrationResponse {
            user,
            email_verification_required,
        }),
    ))
}

pub(super) fn validate_new_password(password: &str, confirm_password: &str) -> BackendResult<()> {
    if password.len() < 8 {
        return Err(anyhow!("Passwords must have at least 8 characters").into());
    }

    if password != confirm_password {
        return Err(anyhow!("Passwords dont match").into());
    }
    Ok(())
}
