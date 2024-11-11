use crate::{
    common::{DbArticle, DbInstance, SearchArticleForm},
    frontend::{app::GlobalState, article_link, article_title},
};
use leptos::*;
use leptos_router::use_query_map;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Default, Clone, Deserialize, Serialize, Debug)]
struct SearchResults {
    articles: Vec<DbArticle>,
    instance: Option<DbInstance>,
}

impl SearchResults {
    pub fn is_empty(&self) -> bool {
        self.articles.is_empty() && self.instance.is_none()
    }
}

#[component]
pub fn Search() -> impl IntoView {
    let params = use_query_map();
    let query = move || params.get().get("query").cloned().unwrap();
    let (error, set_error) = create_signal(None::<String>);
    let search_results = create_resource(query, move |query| async move {
        set_error.set(None);
        let mut search_results = SearchResults::default();
        let api_client = GlobalState::api_client();
        let url = Url::parse(&query);
        let search_data = SearchArticleForm { query };
        let search = api_client.search(&search_data);

        match search.await {
            Ok(mut a) => search_results.articles.append(&mut a),
            Err(e) => set_error.set(Some(e.0.to_string())),
        }

        // If its a valid url, also attempt to resolve as federation object
        if let Ok(url) = url {
            match api_client.resolve_article(url.clone()).await {
                Ok(a) => search_results.articles.push(a.article),
                Err(e) => set_error.set(Some(e.0.to_string())),
            }
            match api_client.resolve_instance(url).await {
                Ok(a) => search_results.instance = Some(a),
                Err(e) => set_error.set(Some(e.0.to_string())),
            }
        }
        search_results
    });

    view! {
        <h1 class="text-4xl font-bold font-serif my-6 grow flex-auto">
            "Search results for " {query}
        </h1>
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || {
                search_results
                    .get()
                    .map(move |search_results| {
                        let is_empty = search_results.is_empty();
                        view! {
                            <Show
                                when=move || !is_empty
                                fallback=move || {
                                    let error_view = move || {
                                        error
                                            .get()
                                            .map(|err| {
                                                view! { <p style="color:red;">{err}</p> }
                                            })
                                    };
                                    view! {
                                        {error_view}
                                        <p>No results found</p>
                                    }
                                }
                            >

                                <ul>

                                    // render resolved instance
                                    {if let Some(instance) = &search_results.instance {
                                        let domain = &instance.domain;
                                        vec![
                                            view! {
                                                <li>
                                                    <a class="link text-lg" href=format!("/instance/{domain}")>
                                                        {domain}
                                                    </a>
                                                </li>
                                            },
                                        ]
                                    } else {
                                        vec![]
                                    }} // render articles from resolve/search
                                    {search_results
                                        .articles
                                        .iter()
                                        .map(|a| {
                                            view! {
                                                <li>
                                                    <a class="link text-lg" href=article_link(a)>
                                                        {article_title(a)}
                                                    </a>
                                                </li>
                                            }
                                        })
                                        .collect::<Vec<_>>()}

                                </ul>
                            </Show>
                        }
                    })
            }}

        </Suspense>
    }
}
