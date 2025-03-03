use super::user::{create_cookie, generate_login_token};
use activitypub_federation::config::Data;
use anyhow::anyhow;
use axum::{Form, Json};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use ibis_api_client::user::{AuthenticateWithOauth, OAuthTokenResponse, RegisterUserParams};
use ibis_database::{
    common::user::LocalUserView,
    config::OAuthProvider,
    error::{BackendError, BackendResult},
    impls::{
        user::{LocalUserViewQuery, OAuthAccount, OAuthAccountInsertForm},
        IbisContext,
    },
};
use ibis_federate::validate::validate_user_name;
use regex::Regex;
use reqwest::Client;
use std::sync::LazyLock;

#[debug_handler]
pub async fn register_user(
    context: Data<IbisContext>,
    jar: CookieJar,
    Form(params): Form<RegisterUserParams>,
) -> BackendResult<(CookieJar, Json<LocalUserView>)> {
    if !context.conf.options.registration_open {
        return Err(anyhow!("Registration is closed").into());
    }

    // Make sure passwords match
    if params.password != params.password_verify {
        Err(anyhow!("PasswordsDoNotMatch"))?;
    }

    if context.conf.options.email_required && params.email.is_none() {
        Err(anyhow!("EmailRequired"))?
    }

    check_new_user(&params.username, params.email.as_deref(), &context)?;

    let user = LocalUserView::create(
        params.username,
        Some(params.password),
        false,
        params.email,
        &context,
    )?;

    check_email_verified(&user, &context)?;

    let token = generate_login_token(&user.person, &context)?;
    let jar = jar.add(create_cookie(token, &context));
    Ok((jar, Json(user)))
}

#[debug_handler]
pub async fn authenticate_with_oauth(
    context: Data<IbisContext>,
    jar: CookieJar,
    Form(params): Form<AuthenticateWithOauth>,
) -> BackendResult<(CookieJar, Json<LocalUserView>)> {
    let oauth_invalid_err: BackendError = anyhow!("OauthAuthorizationInvalid").into();
    // validate inputs
    if params.code.is_empty() || params.code.len() > 300 {
        return Err(oauth_invalid_err);
    }

    // validate the redirect_uri
    let redirect_uri = &params.redirect_uri;
    if redirect_uri.host_str().unwrap_or("").is_empty()
        || !redirect_uri.path().eq(&String::from("/oauth/callback"))
        || !redirect_uri.query().unwrap_or("").is_empty()
    {
        return Err(oauth_invalid_err);
    }

    // validate the PKCE challenge
    if let Some(code_verifier) = &params.pkce_code_verifier {
        check_code_verifier(code_verifier)?;
    }

    // Fetch the OAUTH provider and make sure it's enabled
    let oauth_provider = context
        .conf
        .oauth_providers
        .iter()
        .filter(|provider| provider.enabled)
        .find(|provider| provider.issuer == params.oauth_issuer)
        .ok_or(oauth_invalid_err)?;

    let token_response = oauth_request_access_token(
        &oauth_provider,
        &params.code,
        params.pkce_code_verifier.as_deref(),
        redirect_uri.as_str(),
    )
    .await?;

    let user_info =
        oidc_get_user_info(&oauth_provider, token_response.access_token.as_str()).await?;

    let oauth_user_id = read_user_info(&user_info, oauth_provider.id_claim.as_str())?;

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

        // prevent registration if registration is closed
        if !context.conf.options.registration_open {
            return Err(anyhow!("Registration is closed").into());
        }

        // prevent registration if registration is closed for OAUTH providers
        if !context.conf.options.oauth_registration_open {
            return Err(anyhow!("OAuth registration is closed").into());
        }

        // Extract the OAUTH email claim from the returned user_info
        let email = read_user_info(&user_info, "email")?;

        // Lookup user by OAUTH email and link accounts
        local_user_view = LocalUserView::read(LocalUserViewQuery::Email(&email), &context);

        if let Ok(user_view) = local_user_view {
            // user found by email => link and login if linking is allowed

            // we only allow linking by email when email_verification is required otherwise emails cannot
            // be trusted
            if oauth_provider.account_linking_enabled && context.conf.options.email_required {
                // Link with OAUTH => Login user
                let oauth_account_form = OAuthAccountInsertForm {
                    local_user_id: user_view.local_user.id,
                    oauth_issuer_url: oauth_provider.issuer.clone().into(),
                    oauth_user_id,
                };

                OAuthAccount::create(&oauth_account_form, &context)?;

                user_view
            } else {
                return Err(anyhow!("EmailAlreadyExists"))?;
            }
        } else {
            // No user was found by email => Register as new user

            // make sure the username is provided
            let username = params
                .username
                .ok_or(anyhow!("RegistrationUsernameRequired"))?;

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

    check_email_verified(&user, &context)?;

    let token = generate_login_token(&user.person, &context)?;
    let jar = jar.add(create_cookie(token, &context));
    Ok((jar, Json(user)))
}

static REQWEST: LazyLock<Client> = LazyLock::new(|| Client::new());

async fn oauth_request_access_token(
    oauth_provider: &OAuthProvider,
    code: &str,
    pkce_code_verifier: Option<&str>,
    redirect_uri: &str,
) -> BackendResult<OAuthTokenResponse> {
    let mut form = vec![
        ("client_id", &*oauth_provider.client_id),
        ("client_secret", &*oauth_provider.client_secret),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
    ];

    if let Some(code_verifier) = pkce_code_verifier {
        form.push(("code_verifier", code_verifier));
    }

    // Request an Access Token from the OAUTH provider
    let response = REQWEST
        .post(oauth_provider.token_endpoint.as_str())
        .header("Accept", "application/json")
        .form(&form[..])
        .send()
        .await?
        .error_for_status()?;

    // Extract the access token
    let token_response = response.json::<OAuthTokenResponse>().await?;

    Ok(token_response)
}

async fn oidc_get_user_info(
    oauth_provider: &OAuthProvider,
    access_token: &str,
) -> BackendResult<serde_json::Value> {
    // Request the user info from the OAUTH provider
    let response = REQWEST
        .get(oauth_provider.userinfo_endpoint.as_str())
        .header("Accept", "application/json")
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?;

    // Extract the OAUTH user_id claim from the returned user_info
    let user_info = response.json::<serde_json::Value>().await?;

    Ok(user_info)
}

fn read_user_info(user_info: &serde_json::Value, key: &str) -> BackendResult<String> {
    if let Some(value) = user_info.get(key) {
        let result = serde_json::from_value::<String>(value.clone())?;
        return Ok(result);
    }
    Err(anyhow!("OauthLoginFailed"))?
}

#[allow(clippy::expect_used)]
fn check_code_verifier(code_verifier: &str) -> BackendResult<()> {
    static VALID_CODE_VERIFIER_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9\-._~]{43,128}$").expect("compile regex"));

    let check = VALID_CODE_VERIFIER_REGEX.is_match(code_verifier);

    if check {
        Ok(())
    } else {
        Err(anyhow!("InvalidCodeVerifier").into())
    }
}

fn check_new_user(username: &str, email: Option<&str>, context: &IbisContext) -> BackendResult<()> {
    validate_user_name(username)?;
    LocalUserView::check_username_taken(username, context)?;
    if let Some(email) = email {
        LocalUserView::check_email_taken(email, context)?;
    }
    Ok(())
}

fn check_email_verified(user: &LocalUserView, context: &IbisContext) -> BackendResult<()> {
    if context.conf.options.email_required && user.local_user.email_verified {
        return Err(anyhow!("email not verified").into());
    }
    Ok(())
}
