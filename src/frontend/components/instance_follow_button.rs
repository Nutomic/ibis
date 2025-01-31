use crate::{
    common::{
        instance::{DbInstance, FollowInstanceParams},
        newtypes::InstanceId,
    },
    frontend::{
        api::CLIENT,
        utils::{
            errors::FrontendResultExt,
            resources::{my_profile, site},
        },
    },
};
use leptos::prelude::*;

#[component]
pub fn InstanceFollowButton(instance: DbInstance) -> impl IntoView {
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
    let is_following = my_profile()
        .map(|my_profile| my_profile.following.contains(&instance))
        .unwrap_or(false);
    let follow_text = if is_following {
        "Following instance"
    } else {
        "Follow instance"
    };

    let class_ = if instance.local {
        "hidden"
    } else {
        "btn btn-sm"
    };
    view! {
        <button
            class=class_
            on:click=move |_| {
                follow_action.dispatch(instance.id);
            }
            prop:disabled=move || is_following
            title="Follow the instance so that new edits are synchronized to your instance."
        >
            {follow_text}
        </button>
    }
}
