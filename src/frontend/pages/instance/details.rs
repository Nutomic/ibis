use crate::{
    common::{
        article::ListArticlesParams,
        instance::{GetInstanceParams, Instance},
    },
    frontend::{
        api::CLIENT,
        components::{instance_follow_button::InstanceFollowButton, suspense_error::SuspenseError},
        utils::{
            errors::FrontendError,
            formatting::{
                article_path,
                article_title,
                instance_title_with_domain,
                instance_updated,
            },
        },
    },
};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn InstanceDetails() -> impl IntoView {
    let params = use_params_map();
    let hostname = move || params.get().get("hostname").clone();
    let instance_profile = Resource::new(hostname, move |hostname| async move {
        let hostname = hostname.ok_or(FrontendError::new("No instance given"))?;
        let params = GetInstanceParams {
            id: None,
            hostname: Some(hostname),
        };
        CLIENT.get_instance(&params).await
    });

    view! {
        <SuspenseError result=instance_profile>
            {move || Suspend::new(async move {
                instance_profile
                    .await
                    .map(|i| i.instance)
                    .map(|instance: Instance| {
                        let articles = Resource::new(
                            move || instance.id,
                            |instance_id| async move {
                                CLIENT
                                    .list_articles(ListArticlesParams {
                                        only_local: None,
                                        instance_id: Some(instance_id),
                                    })
                                    .await
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
                                                                        {article_title(&a)}
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
