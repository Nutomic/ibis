use crate::common::{DbInstance, FollowInstance};
use crate::frontend::app::GlobalState;
use leptos::*;
use leptos_router::use_params_map;
use url::Url;

#[component]
pub fn InstanceDetails() -> impl IntoView {
    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
    let params = use_params_map();
    let hostname = move || params.get().get("hostname").cloned().unwrap();
    let instance_profile = create_resource(hostname, move |hostname| async move {
        let url = Url::parse(&format!("http://{hostname}")).unwrap();
        GlobalState::api_client()
            .resolve_instance(url)
            .await
            .unwrap()
    });
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

    view! {
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || instance_profile.get().map(|instance: DbInstance| {
                let instance_ = instance.clone();
                let is_following = global_state.get().my_profile.map(|p| p.following.contains(&instance_)).unwrap_or_default();
                let follow_text = if is_following {
                    "Following"
                } else {
                    "Follow"
                };
                view! {
                    <h1>{instance.domain}</h1>

                    <Show when=move || global_state.with(|state| state.my_profile.is_some())>
                        <button on:click=move |_| follow_action.dispatch(instance.id)
                                prop:disabled=move || is_following>
                            {follow_text}
                        </button>
                    </Show>
                    <p>Follow the instance so that new edits are federated to your instance.</p>
                    <p>"TODO: show a list of articles from the instance. For now you can use the "<a href="/article/list">Article list</a>.</p>
                    <hr/>
                    <h2>"Description:"</h2>
                    <div>{instance.description}</div>
                }
            })
        }</Suspense>
    }
}
