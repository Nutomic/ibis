use crate::{
    common::{DbInstance, FollowInstance},
    frontend::app::GlobalState,
};
use leptos::{component, *};

#[component]
pub fn InstanceFollowButton(instance: DbInstance) -> impl IntoView {
    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
    let follow_action = create_action(move |instance_id: &i32| {
        let instance_id = *instance_id;
        async move {
            let form = FollowInstance { id: instance_id };
            GlobalState::api_client()
                .follow_instance(form)
                .await
                .unwrap();
            GlobalState::update_my_profile();
        }
    });
    let is_following = global_state
        .get_untracked()
        .my_profile
        .map(|p| p.following.contains(&instance))
        .unwrap_or_default();
    let follow_text = if is_following {
        "Following instance"
    } else {
        "Follow instance"
    };

    view! {
      <button
        on:click=move |_| follow_action.dispatch(instance.id)
        prop:disabled=move || is_following
        prop:hidden=move || instance.local
      >
        {follow_text}
      </button>
    }
}
