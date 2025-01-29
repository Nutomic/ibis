use leptos::prelude::*;

#[component]
pub fn SuspenseError<Chil>(children: TypedChildren<Chil>) -> impl IntoView
where
    Chil: IntoView + Send + 'static,
{
    view! {
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            <ErrorBoundary
                fallback=|errors| {
                    view! {
                        <div class="grid h-screen place-items-center">
                            <div class="alert alert-error w-min">
                                {move || {
                                    errors
                                        .get()
                                        .into_iter()
                                        .map(|(_, e)| e.to_string())
                                        .collect::<Vec<_>>()
                                }}
                            </div>
                        </div>
                    }
                }
                children
            ></ErrorBoundary>
        </Suspense>
    }
}
