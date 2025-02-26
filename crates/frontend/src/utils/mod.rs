use chrono::{Duration, Local};
use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use::{SameSite, UseCookieOptions, use_cookie_with_options};

pub mod dark_mode;
pub mod formatting;
pub mod resources;

pub fn use_cookie(name: &str) -> (Signal<Option<bool>>, WriteSignal<Option<bool>>) {
    let expires = (Local::now() + Duration::days(356)).timestamp();
    let cookie_options = UseCookieOptions::default()
        .path("/")
        .expires(expires)
        .same_site(SameSite::Strict);
    use_cookie_with_options::<bool, FromToStringCodec>(name, cookie_options)
}
