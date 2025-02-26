use crate::components::suspense_error::SuspenseError;
use ibis_api_client::{
    CLIENT,
    errors::{FrontendResult, FrontendResultExt},
};
use ibis_database::common::instance::InstanceView;
use leptos::prelude::*;

#[component]
pub fn InstanceFollowButton(instance: Resource<FrontendResult<InstanceView>>) -> impl IntoView {
    let follow_action = Action::new(move |i: &InstanceView| {
        let (id, following) = (i.instance.id, i.following);
        async move {
            CLIENT
                .follow_instance(id, !following)
                .await
                .error_popup(|_| instance.refetch());
        }
    });

    view! {
        <SuspenseError result=instance>
            {move || Suspend::new(async move {
                instance
                    .await
                    .map(|instance_| {
                        let follow_text = if instance_.following { "Unfollow" } else { "Follow" };
                        view! {
                            <button
                                class="btn btn-sm ml-2"
                                on:click=move |_| {
                                    follow_action.dispatch(instance_.clone());
                                }
                                title="Follow the instance so that new edits are synchronized to your instance."
                            >
                                {follow_text}
                            </button>
                        }
                    })
            })}
        </SuspenseError>
    }
}
