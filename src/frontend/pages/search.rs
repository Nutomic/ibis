use crate::common::SearchArticleData;
use crate::frontend::app::GlobalState;
use leptos::*;
use leptos_router::use_query_map;

#[component]
pub fn Search() -> impl IntoView {
    let params = use_query_map();
    let query = params.get_untracked().get("query").cloned().unwrap();
    let query_ = query.clone();
    let search_results = create_resource(
        move || query_.clone(),
        move |query| async move {
            GlobalState::api_client()
                .search(&SearchArticleData { query })
                .await
                .unwrap()
        },
    );

    view! {
        <h1>"Search results for "{query}</h1>
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || search_results.get().map(|search_results| {
                let is_empty = search_results.is_empty();
                view! {
                <Show when=move || !is_empty
                        fallback=|| view! { <p>No results found</p> }>
                    <ul>
                        {
                            search_results
                                .iter()
                                .map(|a| view! { <li>
                                    <a href={format!("/article/{}", a.title)}>{a.title()}</a>
                                </li>})
                                .collect::<Vec<_>>()
                        }
                    </ul>
                </Show>
            }})
        }
        </Suspense>
    }
}
