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
                        <div class="grid place-items-center h-screen">
                            <div class="alert alert-error w-fit">
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
            />
        </Suspense>
    }
}
