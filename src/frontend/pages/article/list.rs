use crate::{
    common::ListArticlesForm,
    frontend::{app::GlobalState, article_link, article_title},
};
use leptos::*;
use web_sys::wasm_bindgen::JsCast;

#[component]
pub fn ListArticles() -> impl IntoView {
    let (only_local, set_only_local) = create_signal(false);
    let articles = create_resource(
        move || only_local.get(),
        |only_local| async move {
            GlobalState::api_client()
                .list_articles(ListArticlesForm {
                    only_local: Some(only_local),
                })
                .await
                .unwrap()
        },
    );

    view! {
        <h1 class="text-4xl font-bold font-serif my-4">Most recently edited Articles</h1>
        <Suspense fallback=|| view! { "Loading..." }>
            <fieldset
                class="flex flex-row"
                on:input=move |ev| {
                    let val = ev
                        .target()
                        .unwrap()
                        .unchecked_into::<web_sys::HtmlInputElement>()
                        .id();
                    let is_local_only = val == "only-local";
                    set_only_local.update(|p| *p = is_local_only);
                }
            >
                <label class="label cursor-pointer max-w-32">
                    <span>Only Local</span>
                    <input type="radio" name="listing-type" class="radio checked:bg-primary" />
                </label>
                <label class="label cursor-pointer max-w-32">
                    <span>All</span>
                    <input
                        type="radio"
                        name="listing-type"
                        class="radio checked:bg-primary"
                        checked="checked"
                    />
                </label>
            </fieldset>
            <ul class="list-disc">
                {move || {
                    articles
                        .get()
                        .map(|a| {
                            a.into_iter()
                                .map(|a| {
                                    view! {
                                        <li>
                                            <a class="link link-secondary" href=article_link(&a)>
                                                {article_title(&a)}
                                            </a>
                                        </li>
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                }}

            </ul>
        </Suspense>
    }
}
