use leptos::prelude::*;
use leptos_fluent::{expect_i18n, leptos_fluent};

#[component]
pub fn I18n(children: Children) -> impl IntoView {
    // See all options in the reference at
    // https://mondeja.github.io/leptos-fluent/leptos_fluent.html

    #[allow(unused_variables)]
    let max_age = 60 * 60 * 24 * 365;
    leptos_fluent! {
        children: children(),
        locales: "../../locales",
        default_language: "en",
        set_language_to_cookie: true,
        initial_language_from_cookie: true,
        initial_language_from_navigator: true,
        initial_language_from_navigator_to_cookie: true,
        initial_language_from_url_param: true,
        initial_language_from_url_param_to_cookie: true,
        initial_language_from_accept_language_header: true,
        cookie_name: "lang",
        cookie_attrs: format!("samesite=strict; path=/; max-age={max_age}"),
    }
}

#[component]
pub(crate) fn LanguageSelector() -> impl IntoView {
    let i18n = expect_i18n();

    view! {
        <select
            class="select select-sm select-neutral"
            prop:value=move || i18n.language.get().id.to_string()
        >
            {move || {
                i18n.languages
                    .iter()
                    .map(|lang| {
                        view! {
                            <option
                                value=lang.id.to_string()
                                on:click=move |_| i18n.language.set(lang)
                            >
                                {lang.name}
                            </option>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </select>
    }
}
