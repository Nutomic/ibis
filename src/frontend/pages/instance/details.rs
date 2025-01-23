use crate::{
    common::{article::ListArticlesParams, instance::DbInstance, utils::http_protocol_str},
    frontend::{
        api::CLIENT,
        article_path,
        article_title,
        components::instance_follow_button::InstanceFollowButton,
        instance_title_with_domain,
        instance_updated,
    },
};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use url::Url;

#[component]
pub fn InstanceDetails() -> impl IntoView {
    let params = use_params_map();
    let hostname = move || params.get().get("hostname").clone().unwrap();
    let instance_profile = Resource::new(hostname, move |hostname| async move {
        let url = Url::parse(&format!("{}://{hostname}", http_protocol_str())).unwrap();
        CLIENT.resolve_instance(url).await.unwrap()
    });

    view! {
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || {
                instance_profile
                    .get()
                    .map(|instance: DbInstance| {
                        let articles = Resource::new(
                            move || instance.id,
                            |instance_id| async move {
                                CLIENT
                                    .list_articles(ListArticlesParams {
                                        only_local: None,
                                        instance_id: Some(instance_id),
                                    })
                                    .await
                                    .unwrap()
                            },
                        );
                        let title = instance_title_with_domain(&instance);
                        let instance_ = instance.clone();
                        view! {
                            <Title text=title.clone() />
                            <div class="grid gap-3 mt-4">
                                <div class="flex flex-row items-center">
                                    <h1 class="w-full font-serif text-4xl font-bold">{title}</h1>
                                    {instance_updated(&instance_)}
                                    <InstanceFollowButton instance=instance_.clone() />
                                </div>

                                <div class="divider"></div>
                                <div>{instance.topic}</div>
                                <h2 class="font-serif text-xl font-bold">Articles</h2>
                                <ul class="list-none">
                                    <Suspense>
                                        {move || {
                                            articles
                                                .get()
                                                .map(|a| {
                                                    a.into_iter()
                                                        .map(|a| {
                                                            view! {
                                                                <li>
                                                                    <a class="text-lg link" href=article_path(&a)>
                                                                        {article_title(&a)}
                                                                    </a>
                                                                </li>
                                                            }
                                                        })
                                                        .collect::<Vec<_>>()
                                                })
                                        }}
                                    </Suspense>
                                </ul>
                            </div>
                        }
                    })
            }}

        </Suspense>
    }
}
