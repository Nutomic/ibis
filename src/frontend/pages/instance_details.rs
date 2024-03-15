use crate::{
    common::{utils::http_protocol_str, DbInstance},
    frontend::{app::GlobalState, components::instance_follow_button::InstanceFollowButton},
};
use leptos::*;
use leptos_router::use_params_map;
use url::Url;

#[component]
pub fn InstanceDetails() -> impl IntoView {
    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
    let params = use_params_map();
    let hostname = move || params.get().get("hostname").cloned().unwrap();
    let instance_profile = create_resource(hostname, move |hostname| async move {
        let url = Url::parse(&format!("{}://{hostname}", http_protocol_str())).unwrap();
        GlobalState::api_client()
            .resolve_instance(url)
            .await
            .unwrap()
    });

    view! {
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || instance_profile.get().map(|instance: DbInstance| {
                let instance_ = instance.clone();
                view! {
                    <h1>{instance.domain}</h1>

                    <Show when=move || global_state.with(|state| state.my_profile.is_some())>
                        <InstanceFollowButton instance=instance_.clone() />
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
