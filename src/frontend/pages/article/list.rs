use crate::{
    common::article::ListArticlesParams,
    frontend::{
        api::CLIENT,
        app::DefaultResource,
        article_path,
        article_title,
        components::connect::ConnectView,
    },
};
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn ListArticles() -> impl IntoView {
    let (only_local, set_only_local) = signal(false);
    let articles = Resource::new(
        move || only_local.get(),
        |only_local| async move {
            CLIENT
                .list_articles(ListArticlesParams {
                    only_local: Some(only_local),
                    instance_id: None,
                })
                .await
        },
    );
    let only_local_class = Resource::new(
        move || only_local.get(),
        |only_local| async move {
            if only_local {
                "btn rounded-r-none btn-primary"
            } else {
                "btn rounded-r-none"
            }
            .to_string()
        },
    );
    let all_class = Resource::new(
        move || only_local.get(),
        |only_local| async move {
            if !only_local {
                "btn rounded-l-none btn-primary"
            } else {
                "btn rounded-l-none"
            }
            .to_string()
        },
    );

    view! {
        <Title text="Recently edited Articles" />
        <h1 class="my-4 font-serif text-4xl font-bold">"Recently edited Articles"</h1>
        <Suspense fallback=|| view! { "Loading..." }>
            <div class="divide-x">
                <input
                    type="button"
                    value="Only Local"
                    class=move || only_local_class.get()
                    on:click=move |_| {
                        set_only_local.set(true);
                    }
                />
                <input
                    type="button"
                    value="All"
                    class=move || all_class.get()
                    on:click=move |_| {
                        set_only_local.set(false);
                    }
                />
            </div>
            <Show
                when=move || {
                    articles.get_default().unwrap_or_default().len() > 1 || only_local.get()
                }
                fallback=move || view! { <ConnectView res=articles /> }
            >
                <ul class="my-4 list-none">
                    <For
                        each=move || articles.get_default().unwrap_or_default()
                        key=|article| article.id
                        let:article
                    >
                        {
                            view! {
                                <li>
                                    <a class="text-lg link" href=article_path(&article)>
                                        {article_title(&article)}
                                    </a>
                                </li>
                            }
                        }
                    </For>

                </ul>
            </Show>
        </Suspense>
    }
}
