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

    view! {
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || instance_profile.get().map(|instance: DbInstance| {
                view! {
                    <h1>{instance.ap_id.to_string()}</h1>
                    <button text="Follow"/>
                    <p>Follow the instance so that new edits are federated to your instance.</p>
                    <div>{instance.description}</div>
                    <p>TODO: show a list of articles from the instance. For now you can use the <a href="/article/list">Article list</a>.</p>
                }
            })
        }</Suspense>
    }
}
