use crate::frontend::{
    pages::article_title_param,
    utils::{errors::FrontendResult, resources::is_logged_in},
};
use leptos::{either::Either, prelude::*};

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
                    let article_title = article_title_param();
                    let href = format!(
                        "/create-article?title={}",
                        article_title.clone().unwrap_or_default(),
                    );
                    Either::Left(
                        view! {
                            <div class="grid place-items-center h-screen">
                                <div>
                                    <div class="alert alert-error w-fit">{e.message()}</div>
                                    <Show when=move || article_title.is_some() && is_logged_in()>
                                        <a class="mt-4 btn" href=href.clone()>
                                            Create Article
                                        </a>
                                    </Show>
                                </div>
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
