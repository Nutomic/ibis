use leptos::{either::Either, prelude::*};

use crate::frontend::utils::errors::FrontendResult;

#[component]
pub fn SuspenseError<T>(children: ChildrenFn, result: Resource<FrontendResult<T>>) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    view! {
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || {
                if let Some(Err(e)) = result.get() {
                    Either::Left(
                        view! {
                            <div class="grid place-items-center h-screen">
                                <div class="alert alert-error w-fit">{e.0}</div>
                            </div>
                        },
                    )
                } else {
                    Either::Right(children())
                }
            }}

        </Suspense>
    }
}
