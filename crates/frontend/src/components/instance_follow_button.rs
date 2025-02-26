use crate::utils::resources::site;
use ibis_api_client::{CLIENT, errors::FrontendResultExt, instance::FollowInstanceParams};
use ibis_database::common::{instance::InstanceView, newtypes::InstanceId};
use leptos::prelude::*;

#[component]
pub fn InstanceFollowButton(instance: InstanceView) -> impl IntoView {
    let follow_action = Action::new(move |instance_id: &InstanceId| {
        let instance_id = *instance_id;
        async move {
            let params = FollowInstanceParams { id: instance_id };
            CLIENT
                .follow_instance(params)
                .await
                .error_popup(|_| site().refetch());
        }
    });
    let follow_text = if instance.following {
        "Following instance"
    } else {
        "Follow instance"
    };

    view! {
        <button
            class="btn btn-sm ml-2"
            on:click=move |_| {
                follow_action.dispatch(instance.instance.id);
            }
            prop:disabled=move || instance.following
            title="Follow the instance so that new edits are synchronized to your instance."
        >
            {follow_text}
        </button>
    }
}
