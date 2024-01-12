use crate::frontend::api::get_article;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Article() -> impl IntoView {
    let params = use_params_map();
    let article = create_resource(
        move || {
            params
                .get()
                .get("title")
                .cloned()
                .unwrap_or("Main Page".to_string())
        },
        move |title| async move { get_article("localhost:8131", title).await.unwrap() },
    );

    view! {
        <Suspense fallback=|| view! {  "Loading..." }>
            {move || article.get().map(|article|
                view! {
                    <div class="item-view">
                        <h1>{article.article.title}</h1>
                        <div>{article.article.text}</div>
                    </div>
            })}
        </Suspense>
    }
}
