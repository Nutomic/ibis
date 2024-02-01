use crate::common::{DbArticle, DbInstance, SearchArticleData};
use crate::frontend::app::GlobalState;
use futures::join;
use leptos::*;
use leptos_router::use_query_map;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Default, Clone, Deserialize, Serialize)]
struct SearchResults {
    articles: Vec<DbArticle>,
    instance: Option<DbInstance>,
}

#[component]
pub fn Search() -> impl IntoView {
    let params = use_query_map();
    let query = move || params.get().get("query").cloned().unwrap();
    let search_results = create_resource(query, move |query| async move {
        let mut search_results = SearchResults::default();
        let api_client = GlobalState::api_client();
        let url = Url::parse(&query);
        let search_data = SearchArticleData { query };
        let search = api_client.search(&search_data);

        // If its a valid url, also attempt to resolve as federation object
        if let Ok(url) = url {
            let resolve_article = api_client.resolve_article(url.clone());
            let resolve_instance = api_client.resolve_instance(url);
            let (search, resolve_article, resolve_instance) =
                join!(search, resolve_article, resolve_instance);
            search_results.instance = resolve_instance.ok();
            if let Ok(article) = resolve_article {
                search_results.articles.push(article.article);
            }
            search_results.articles.append(&mut search.unwrap())
        } else {
            search_results.articles.append(&mut search.await.unwrap())
        }
        search_results
    });

    view! {
        <h1>"Search results for "{query}</h1>
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || search_results.get().map(|search_results| {
                let is_empty = search_results.articles.is_empty() && search_results.instance.is_none();
                view! {
                <Show when=move || !is_empty
                        fallback=|| view! { <p>No results found</p> }>
                    <ul>
                        {
                            // render resolved instance
                            if let Some(instance) = &search_results.instance {
                                let ap_id = instance.ap_id.to_string();
                                vec![view! { <li>
                                    <a href={format!("/instance/{ap_id}")}>{ap_id}</a>
                                </li>}]
                            } else { vec![] }
                        }
                        {
                            // render articles from resolve/search
                            search_results.articles
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
