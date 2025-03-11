use chrono::{Duration, Local};
use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use::{SameSite, UseCookieOptions, use_cookie_with_options};
use std::str::FromStr;

pub mod dark_mode;
pub mod formatting;
pub mod resources;

pub fn use_cookie<T>(name: &str) -> (Signal<Option<T>>, WriteSignal<Option<T>>)
where
    T: Send + Sync + FromStr + ToString + Clone + 'static,
{
    let expires = (Local::now() + Duration::days(356)).timestamp();
    let cookie_options = UseCookieOptions::default()
        .path("/")
        .expires(expires)
        .same_site(SameSite::Strict);
    use_cookie_with_options::<T, FromToStringCodec>(name, cookie_options)
}
