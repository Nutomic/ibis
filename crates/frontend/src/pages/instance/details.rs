use ibis_api_client::{
    CLIENT,
    article::ListArticlesParams,
    errors::FrontendError,
    instance::GetInstanceParams,
};
use ibis_frontend_components::{
    instance_follow_button::InstanceFollowButton,
    suspense_error::SuspenseError,
    utils::formatting::{article_path, instance_title_with_domain, instance_updated},
};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use phosphor_leptos::{ARROW_SQUARE_OUT, Icon};

#[component]
pub fn InstanceDetails() -> impl IntoView {
    let params = use_params_map();
    let hostname = move || params.get().get("hostname").clone();
    let instance = Resource::new(hostname, move |hostname| async move {
        let hostname = hostname.ok_or(FrontendError::new("No instance given"))?;
        let params = GetInstanceParams {
            id: None,
            hostname: Some(hostname),
        };
        CLIENT.get_instance(&params).await
    });

    view! {
        <SuspenseError result=instance>
            {move || Suspend::new(async move {
                instance
                    .await
                    .map(|instance_| {
                        let articles = Resource::new(
                            move || instance_.instance.id,
                            |instance_id| async move {
                                CLIENT
                                    .list_articles(ListArticlesParams {
                                        only_local: None,
                                        instance_id: Some(instance_id),
                                        include_removed: None,
                                    })
                                    .await
                            },
                        );
                        let title = instance_title_with_domain(&instance_.instance);
                        let local = !instance_.instance.local;
                        let ap_id = instance_.instance.ap_id.to_string();
                        view! {
                            <Title text=title.clone() />
                            <div class="grid gap-3 mt-4">
                                <div class="flex flex-row items-center">
                                    <h1 class="font-serif text-4xl font-bold shrink mr-2">
                                        {title}
                                    </h1>
                                    <Show when=move || local>
                                        <a href=ap_id.clone()>
                                            <Icon
                                                icon=ARROW_SQUARE_OUT
                                                size="2.5rem"
                                                {..}
                                                class="p-1"
                                            />
                                        </a>
                                    </Show>
                                    <div class="grow"></div>
                                    {instance_updated(&instance_)}
                                    <InstanceFollowButton instance=instance />
                                </div>

                                <div class="divider"></div>
                                <div>{instance_.instance.topic}</div>
                                <h2 class="font-serif text-xl font-bold">Articles</h2>
                                <ul class="list-none">
                                    <SuspenseError result=articles>
                                        {move || Suspend::new(async move {
                                            articles
                                                .await
                                                .map(|a| {
                                                    a.into_iter()
                                                        .map(|a| {
                                                            view! {
                                                                <li>
                                                                    <a class="text-lg link" href=article_path(&a)>
                                                                        {a.title()}
                                                                    </a>
                                                                </li>
                                                            }
                                                        })
                                                        .collect::<Vec<_>>()
                                                })
                                        })}
                                    </SuspenseError>
                                </ul>
                            </div>
                        }
                    })
            })}

        </SuspenseError>
    }
}
