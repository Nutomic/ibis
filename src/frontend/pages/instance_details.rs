use crate::common::DbInstance;
use crate::frontend::app::GlobalState;
use leptos::*;
use leptos_router::use_params_map;
use url::Url;

#[component]
pub fn InstanceDetails() -> impl IntoView {
    let params = use_params_map();
    let hostname = move || params.get().get("hostname").cloned().unwrap();
    let instance_profile = create_resource(hostname, move |hostname| async move {
        let url = Url::parse(&format!("http://{hostname}")).unwrap();
        GlobalState::api_client()
            .resolve_instance(url)
            .await
            .unwrap()
    });

    // TODO: display list of articles from instance?
    view! {
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || instance_profile.get().map(|instance: DbInstance| {
                view! {
                    <h1>{instance.ap_id.to_string()}</h1>
                    <Show when=GlobalState::is_admin()>
                        <button text="Follow"/>
                    </Show>
                    <div>{instance.description}</div>
                }
            })
        }</Suspense>
    }
}
