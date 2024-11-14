use crate::{
    common::{newtypes::InstanceId, DbInstance, FollowInstance},
    frontend::{
        api::CLIENT,
        app::{site, DefaultResource},
    },
};
use leptos::{component, *};

#[component]
pub fn InstanceFollowButton(instance: DbInstance) -> impl IntoView {
    let follow_action = create_action(move |instance_id: &InstanceId| {
        let instance_id = *instance_id;
        async move {
            let form = FollowInstance { id: instance_id };
            CLIENT.follow_instance(form).await.unwrap();
            site().refetch();
        }
    });
    let is_following = site()
        .with_default(|site| {
            site.clone()
                .my_profile
                .map(|p| p.following.contains(&instance))
        })
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
            on:click=move |_| follow_action.dispatch(instance.id)
            prop:disabled=move || is_following
            title="Follow the instance so that new edits are synchronized to your instance."
        >
            {follow_text}
        </button>
    }
}
