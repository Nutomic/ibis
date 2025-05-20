use ibis_api_client::{
    CLIENT,
    errors::{ErrorPopup, FrontendResultExt},
    user::AuthenticateWithOauth,
};
use ibis_frontend_components::{oauth_login_button::OauthCookie, utils::use_cookie};
use leptos::{prelude::*, task::spawn};
use leptos_router::hooks::use_query_map;

#[component]
pub fn OauthCallback() -> impl IntoView {
    use_cookie("oauth_state")
        .0
        .with(|cookie: &Option<OauthCookie>| {
            let Some(cookie) = cookie.clone() else { return };
            let query_map = use_query_map().get_untracked();
            let state = query_map.get("state");
            let code = query_map.get("code");
            if state != Some(cookie.state) || code.is_none() {
                ErrorPopup::set("OAuth cookie is invalid".to_string());
                let uri = format!("{}/login", location().origin().unwrap_or_default());
                window().location().set_href(&uri).expect("set location");
            }

            let params = AuthenticateWithOauth {
                code: code.expect("code is set"),
                oauth_issuer: cookie.issuer_url,
                redirect_uri: cookie.redirect_url,
                username: cookie.username,
            };
            spawn(async move {
                CLIENT
                    .oauth_authenticate(params)
                    .await
                    .error_popup(|_| window().location().set_pathname("/").expect("set location"));
            });
        });

    view! { Loading... }
}
