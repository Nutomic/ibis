use crate::{
    components::suspense_error::SuspenseError,
    utils::{resources::site, use_cookie},
};
use ibis_database::common::instance::OAuthProviderPublic;
use leptos::{ev::MouseEvent, prelude::*};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
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

#[component]
pub fn OauthLoginButtons(username: ReadSignal<String>) -> impl IntoView {
    let site = site();
    view! {
        <SuspenseError result=site>
            {move || Suspend::new(async move {
                let site = site.await;
                let providers: Vec<_> = site
                    .as_ref()
                    .ok()
                    .map(|s| s.oauth_providers.clone())
                    .into_iter()
                    .flatten()
                    .collect();
                let has_oauth_providers = !providers.is_empty();
                view! {
                    <Show when=move || { has_oauth_providers }>
                        <h2 class="my-4 font-serif text-xl font-bold grow max-w-fit">
                            Or Register with SSO Provider
                        </h2>
                        {providers
                            .iter()
                            .map(|p| {
                                view! {
                                    <button
                                        class="m-2 btn btn-secondary"
                                        on:click=on_click(p.clone(), username)
                                    >
                                        {p.display_name.clone()}
                                    </button>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </Show>
                }
            })}
        </SuspenseError>
    }
}

fn on_click(provider: OAuthProviderPublic, username: ReadSignal<String>) -> impl Fn(MouseEvent) {
    let oauth_cookie = use_cookie("oauth_state");
    move |_| {
        let redirect_uri = Url::parse(&format!(
            "{}/account/oauth_callback",
            location().origin().expect("get location")
        ))
        .expect("format url");
        let state = Uuid::new_v4().to_string();

        oauth_cookie.1.set(Some(OauthCookie {
            state: state.clone(),
            issuer_url: provider.issuer.clone(),
            redirect_url: redirect_uri.clone(),
            username: Some(username.get()),
        }));

        let mut oauth_redirect = provider.authorization_endpoint.clone();
        oauth_redirect
            .query_pairs_mut()
            .append_pair("client_id", &provider.client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", &provider.scopes)
            .append_pair("redirect_uri", redirect_uri.as_str())
            .append_pair("state", &state)
            .finish();
        window()
            .location()
            .set_href(oauth_redirect.as_str())
            .expect("set location")
    }
}
