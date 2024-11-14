use leptos::{component, *};
use url::Url;use crate::frontend::api::CLIENT;

#[component]
pub fn ConnectView<T: Clone + 'static, R: 'static>(res: Resource<T, R>) -> impl IntoView {
    let connect_ibis_wiki = create_action(move |_: &()| async move {
        CLIENT
            .resolve_instance(Url::parse("https://ibis.wiki").unwrap())
            .await
            .unwrap();
        res.refetch();
    });

    view! {
        <div class="flex justify-center h-screen">
            <button
                class="btn btn-primary place-self-center"
                on:click=move |_| connect_ibis_wiki.dispatch(())
            >
                Connect with ibis.wiki
            </button>
        </div>
    }
}
