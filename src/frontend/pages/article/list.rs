use crate::{
    common::ListArticlesForm,
    frontend::{app::GlobalState, article_link, article_title, components::connect::ConnectView},
};
use html::Input;
use leptos::*;

#[component]
pub fn ListArticles() -> impl IntoView {
    let (only_local, set_only_local) = create_signal(false);
    let button_only_local = create_node_ref::<Input>();
    let button_all = create_node_ref::<Input>();
    let articles = create_resource(
        move || only_local.get(),
        |only_local| async move {
            GlobalState::api_client()
                .list_articles(ListArticlesForm {
                    only_local: Some(only_local),
                    instance_id: None,
                })
                .await
                .unwrap()
        },
    );

    view! {
        <h1 class="text-4xl font-bold font-serif my-4">Most recently edited Articles</h1>
        <Suspense fallback=|| view! { "Loading..." }>
            <div class="divide-x">
                <input
                    type="button"
                    value="Only Local"
                    class="btn rounded-r-none"
                    node_ref=button_only_local
                    on:click=move |_| {
                        button_all.get().map(|c| c.class("btn-primary", false));
                        button_only_local.get().map(|c| c.class("btn-primary", true));
                        set_only_local.set(true);
                    }
                />
                <input
                    type="button"
                    value="All"
                    class="btn btn-primary rounded-l-none"
                    node_ref=button_all
                    on:click=move |_| {
                        button_all.get().map(|c| c.class("btn-primary", true));
                        button_only_local.get().map(|c| c.class("btn-primary", false));
                        set_only_local.set(false);
                    }
                />
            </div>
            <Show
                when=move || { articles.get().unwrap_or_default().len() > 1 || only_local.get() }
                fallback=move || view! { <ConnectView res=articles /> }
            >
                <ul class="list-none my-4">
                    {move || {
                        articles
                            .get()
                            .map(|a| {
                                a.into_iter()
                                    .map(|a| {
                                        view! {
                                            <li>
                                                <a class="link text-lg" href=article_link(&a)>
                                                    {article_title(&a)}
                                                </a>
                                            </li>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            })
                    }}

                </ul>
            </Show>
        </Suspense>
    }
}
