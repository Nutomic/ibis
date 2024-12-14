use crate::frontend::api::CLIENT;
use codee::{Decoder, Encoder};
use leptos::prelude::*;
use std::fmt::Debug;
use url::Url;

#[component]
pub fn ConnectView<T, R>(res: Resource<T, R>) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
    R: Encoder<T> + Decoder<T> + Send + Sync + 'static,
    <R as Encoder<T>>::Error: Debug,
    <R as Encoder<T>>::Encoded: IntoEncodedString,
    <R as Decoder<T>>::Encoded: FromEncodedStr,
    <R as Decoder<T>>::Error: Debug,
    <<R as Decoder<T>>::Encoded as leptos::prelude::FromEncodedStr>::DecodingError: Debug,
{
    let connect_ibis_wiki = Action::new(move |_: &()| async move {
        CLIENT
            .resolve_instance(Url::parse("https://ibis.wiki").unwrap())
            .await
            .unwrap();
        res.refetch();
    });

    view! {
        <div class="flex justify-center h-screen">
            <button
                class="place-self-center btn btn-primary"
                on:click=move |_| {
                    connect_ibis_wiki.dispatch(());
                }
            >
                Connect with ibis.wiki
            </button>
        </div>
    }
}
