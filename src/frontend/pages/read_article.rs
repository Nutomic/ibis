use crate::common::GetArticleData;
use crate::frontend::app::GlobalState;
use leptos::*;
use leptos_router::*;

#[component]
pub fn ReadArticle() -> impl IntoView {
    let params = use_params_map();
    let article = create_resource(
        move || {
            params
                .get()
                .get("title")
                .cloned()
                .unwrap_or("Main_Page".to_string())
        },
        move |title| async move {
            GlobalState::api_client()
                .get_article(GetArticleData {
                    title: Some(title),
                    instance_id: None,
                    id: None,
                })
                .await
                .unwrap()
        },
    );

    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
    let (count, set_count) = create_signal(0);
    view! {
        <Suspense fallback=|| view! {  "Loading..." }>
            {move || article.get().map(|article|
                view! {
                    <div class="item-view">
                        <h1>{article.article.title}</h1>
                        <Show when=move || global_state.with(|state| state.my_profile.is_some())>
                            <button on:click=move |_| {
                                set_count.update(|n| *n += 1);
                            }>Edit {move || count.get()}</button>
                        </Show>
                        <div>{article.article.text}</div>
                    </div>
            })}
        </Suspense>
    }
}
