use crate::{
    components::suspense_error::SuspenseError,
    utils::formatting::{article_link, instance_title_with_domain, instance_updated},
};
use codee::string::JsonSerdeCodec;
use ibis_api_client::{
    CLIENT,
    errors::{FrontendResult, FrontendResultExt},
};
use ibis_database::common::{article::Article, instance::InstanceWithArticles};
use leptos::prelude::*;
use leptos_meta::Title;
use url::Url;

#[component]
pub fn Explore() -> impl IntoView {
    let instances = Resource::new(move || (), |_| async move { CLIENT.list_instances().await });

    view! {
        <Title text="Explore" />
        <h1 class="my-4 font-serif text-4xl font-bold">Instances</h1>
        <SuspenseError result=instances>
            {move || Suspend::new(async move {
                let instances_ = instances.await;
                let is_empty = instances_.as_ref().map(|i| i.is_empty()).unwrap_or(true);
                view! {
                    <Show
                        when=move || !is_empty
                        fallback=move || view! { <ConnectView res=instances /> }
                    >
                        <ul class="my-4 list-none">
                            {instances_
                                .clone()
                                .ok()
                                .into_iter()
                                .flatten()
                                .map(instance_card)
                                .collect::<Vec<_>>()}
                        </ul>
                    </Show>
                }
            })}
        </SuspenseError>
    }
}

pub fn instance_card(i: InstanceWithArticles) -> impl IntoView {
    view! {
        <li>
            <div class="my-4 shadow card bg-base-100">
                <div class="p-4 card-body">
                    <div class="flex">
                        <a class="card-title grow" href=format!("/instance/{}", i.instance.domain)>
                            {instance_title_with_domain(&i.instance)}
                        </a>
                        {instance_updated(&i.instance)}
                    </div>
                    <p>{i.instance.topic.clone()}</p>
                    <div class="flex flex-col text-base/5">
                        <For
                            each=move || i.articles.clone()
                            key=|article| article.id
                            children=move |article: Article| {
                                view! { {article_link(&article)} }
                            }
                        />
                    </div>
                </div>
            </div>
        </li>
    }
}

#[component]
fn ConnectView(
    res: Resource<FrontendResult<Vec<InstanceWithArticles>>, JsonSerdeCodec>,
) -> impl IntoView {
    let connect_ibis_wiki = Action::new(move |_: &()| async move {
        CLIENT
            .resolve_instance(Url::parse("https://ibis.wiki").expect("parse ibis.wiki url"))
            .await
            .error_popup(|_| res.refetch());
    });

    view! {
        <div class="flex justify-center h-screen">
            <button
                class="place-self-center btn btn-primary"
                on:click=move |_| {
                    connect_ibis_wiki.dispatch(());
                }
            >
                Connect with ibis.wiki
            </button>
        </div>
    }
}
