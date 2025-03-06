use crate::utils::use_cookie;
use ibis_database::common::instance::OAuthProviderPublic;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::{ops::Deref, str::FromStr, sync::Arc};
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct OauthCookie {
    pub(crate) state: String,
    pub(crate) issuer_url: Url,
    pub(crate) redirect_url: Url,
    pub(crate) username: Option<String>,
}

impl FromStr for OauthCookie {
    type Err = serde_json::Error;
    fn from_str(val: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(val)
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for OauthCookie {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("convert oauth cookie to string")
    }
}

pub fn oauth_login_button(
    provider: OAuthProviderPublic,
    username: Option<String>,
) -> impl IntoView {
    let provider_ = Arc::new(provider.clone());
    let username = Arc::new(username.clone());
    let oauth_cookie = use_cookie("oauth_state");
    let on_click = move |_| {
        let redirect_uri = Url::parse(&format!(
            "{}/account/oauth_callback",
            location().origin().expect("get location")
        ))
        .expect("format url");
        let state = Uuid::new_v4().to_string();

        oauth_cookie.1.set(Some(OauthCookie {
            state: state.clone(),
            issuer_url: provider_.issuer.clone(),
            redirect_url: redirect_uri.clone(),
            username: username.deref().clone(),
        }));

        let mut oauth_redirect = provider_.authorization_endpoint.clone();
        oauth_redirect
            .query_pairs_mut()
            .append_pair("client_id", &provider_.client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", &provider_.scopes)
            .append_pair("redirect_uri", redirect_uri.as_str())
            .append_pair("state", &state)
            .finish();
        window()
            .location()
            .set_href(oauth_redirect.as_str())
            .expect("set location")
    };

    view! {
        <button class="my-2 btn btn-secondary" on:click=on_click>
            {provider.display_name.clone()}
        </button>
    }
}
