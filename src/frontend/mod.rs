use crate::common::{utils::extract_domain, DbArticle, DbPerson};
use chrono::{DateTime, Duration, Local, Utc};
use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};

pub mod api;
pub mod app;
mod components;
pub mod dark_mode;
pub mod markdown;
pub mod pages;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::frontend::app::App;
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

fn article_path(article: &DbArticle) -> String {
    if article.local {
        format!("/article/{}", article.title)
    } else {
        format!(
            "/article/{}@{}",
            article.title,
            extract_domain(&article.ap_id)
        )
    }
}

fn article_link(article: &DbArticle) -> impl IntoView {
    let article_path = article_path(article);
    view! {
        <a class="link" href=article_path>
            {article.title.clone()}
        </a>
    }
}

fn article_title(article: &DbArticle) -> String {
    let title = article.title.replace('_', " ");
    if article.local {
        title
    } else {
        format!("{}@{}", title, extract_domain(&article.ap_id))
    }
}

fn user_title(person: &DbPerson) -> String {
    let name = person
        .display_name
        .clone()
        .unwrap_or(person.username.clone());
    if person.local {
        name.clone()
    } else {
        format!("{}@{}", name, extract_domain(&person.ap_id))
    }
}

fn user_link(person: &DbPerson) -> impl IntoView {
    let creator_path = if person.local {
        format!("/user/{}", person.username)
    } else {
        format!(
            "/user/{}@{}",
            person.username,
            extract_domain(&person.ap_id)
        )
    };
    view! {
        <a class="link" href=creator_path>
            {user_title(person)}
        </a>
    }
}

fn render_date_time(date_time: DateTime<Utc>) -> String {
    date_time
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn use_cookie(name: &str) -> (Signal<Option<bool>>, WriteSignal<Option<bool>>) {
    let expires = (Local::now() + Duration::days(356)).timestamp();
    let cookie_options = UseCookieOptions::default()
        .path("/")
        .expires(expires)
        .same_site(SameSite::Strict);
    use_cookie_with_options::<bool, FromToStringCodec>(name, cookie_options)
}
