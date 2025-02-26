use crate::utils::formatting::{article_path, article_title};
use ibis_api_client::{CLIENT, instance::SearchArticleParams};
use ibis_database::common::{article::Article, instance::Instance};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Default, Clone, Deserialize, Serialize, Debug)]
struct SearchResults {
    articles: Vec<Article>,
    instance: Option<Instance>,
}

impl SearchResults {
    pub fn is_empty(&self) -> bool {
        self.articles.is_empty() && self.instance.is_none()
    }
}

#[component]
pub fn Search() -> impl IntoView {
    let params = use_query_map();
    let (error, set_error) = signal(None::<String>);
    let search_results = Resource::new(
        move || params.get().get("query").unwrap_or_default(),
        move |query| async move {
            set_error.set(None);
            let mut search_results = SearchResults::default();
            let url = Url::parse(&query);
            let search_data = SearchArticleParams { query };
            let search = CLIENT.search(&search_data);

            match search.await {
                Ok(mut a) => search_results.articles.append(&mut a),
                Err(e) => set_error.set(Some(e.to_string())),
            }

            // If its a valid url, also attempt to resolve as federation object
            if let Ok(url) = url {
                match CLIENT.resolve_article(url.clone()).await {
                    Ok(a) => search_results.articles.push(a.article),
                    Err(e) => set_error.set(Some(e.to_string())),
                }
                match CLIENT.resolve_instance(url).await {
                    Ok(a) => search_results.instance = Some(a),
                    Err(e) => set_error.set(Some(e.to_string())),
                }
            }
            search_results
        },
    );

    view! {
        <Title text=move || format!("Search - {}", params.get().get("query").unwrap_or_default()) />
        <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">
            "Search results for " {move || params.get().get("query").unwrap_or_default()}
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
                                                    <a class="text-lg link" href=format!("/instance/{domain}")>
                                                        {domain.to_string()}
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
                                                    <a class="text-lg link" href=article_path(a)>
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
